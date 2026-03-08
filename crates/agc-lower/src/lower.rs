//! Lowering: AGC [`SemOp`] sequences → TC2 stack bytecode.
//!
//! All ones-complement operations are expressed as sequences of TC2 instructions.
//! No OC-specific arithmetic opcodes appear in the output — only the pure TC2 ops
//! defined in [`crate::bytecode`] and the bit-pattern predicates that check OC
//! representations without performing OC arithmetic.
//!
//! # Helper sequences emitted for OC operations
//!
//! ## `oc_add(a, b)` — end-around carry addition
//! ```text
//! raw = (a & 0x7FFF) + (b & 0x7FFF)   // TC2 ADD of two 15-bit values
//! carry = raw >> 15                     // extracts the carry-out of bit 15
//! result = (raw + carry) & 0x7FFF       // fold carry back in, mask to 15 bits
//! ```
//!
//! ## `oc_neg(x)` — ones-complement negation
//! ```text
//! result = (!x) & 0x7FFF               // TC2 NOT + MASK15
//! ```
//!
//! ## `oc_sub(a, b)` — subtraction via `oc_add(a, oc_neg(b))`
//!
//! ## `oc_sat(a)` — saturate 16-bit A (overflow → ±max)
//! Detects overflow via `bit14 XOR bit15`; emits conditional with JUMP_NOT/JUMP.
//!
//! ## `oc_aug(w)` — augment away from zero
//! Zero cases short-circuit; sign determined by bit 14; adds ±1 in OC.
//!
//! ## `oc_dim(w)` — diminish toward zero
//! Zero cases short-circuit; handles ±1 boundary; subtracts ±1 in OC.
//!
//! ## `mul_hi/lo(a, b)` — OC multiply
//! Converts both operands OC → TC2 signed, uses `IMUL_HI15`/`IMUL_LO15`,
//! then converts the result TC2 signed → OC.
//!
//! ## `div_q/r(a_hi, a_lo, divisor)` — OC divide
//! Converts operands OC → TC2, uses `IDIV_Q15`/`IDIV_R15`, converts back.
//!
//! ## `dp_add_hi(a_hi, a_lo, b_hi, b_lo)` — double-precision add high word
//! Computes `oc_add(a_lo, b_lo)`, extracts carry via TC2 arithmetic,
//! then `oc_add(oc_add(a_hi, b_hi), carry)`.

use agc_isa::{Cond, Dest, Expr, Flag, SemOp};
use crate::bytecode::Instr;
use crate::vm::addr;

// ─── Public entry point ───────────────────────────────────────────────────────

/// Lower a [`SemOp`] sequence to TC2 bytecode.
///
/// `operand` is the effective address extracted from the instruction word;
/// it is loaded into the VM's `OFF` register at the start of execution.
///
/// The returned `Vec<u16>` is ready to be passed to [`crate::vm::BytecodeVm::exec`].
pub fn lower_sem(operand: u16, ops: &[SemOp]) -> Vec<u16> {
    let mut b = Builder::new();
    b.emit(Instr::SetOff(operand));
    for op in ops {
        lower_op(&mut b, op);
    }
    b.emit(Instr::Ret);
    b.finish()
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Bytecode builder with forward-jump patching support.
struct Builder {
    words: Vec<u16>,
}

impl Builder {
    fn new() -> Self { Builder { words: Vec::new() } }

    fn emit(&mut self, instr: Instr) { instr.encode(&mut self.words); }

    /// Emit `JUMP` with a zero placeholder offset; return the position of the opcode
    /// word so it can be patched later with [`patch`].
    fn jump_placeholder(&mut self) -> usize {
        let pos = self.words.len();
        Instr::Jump(0).encode(&mut self.words);
        pos
    }

    /// Emit `JUMP_IF` (jump if non-zero) with placeholder; return opcode position.
    fn jump_if_placeholder(&mut self) -> usize {
        let pos = self.words.len();
        Instr::JumpIf(0).encode(&mut self.words);
        pos
    }

    /// Emit `JUMP_NOT` (jump if zero) with placeholder; return opcode position.
    fn jump_not_placeholder(&mut self) -> usize {
        let pos = self.words.len();
        Instr::JumpNot(0).encode(&mut self.words);
        pos
    }

    /// Patch the jump at `pos` to target the current end of the word stream.
    ///
    /// The operand of any two-word jump instruction is at `pos + 1`.
    /// The offset is relative to the word *after* the two-word instruction,
    /// so `offset = current_len - (pos + 2)`.
    fn patch(&mut self, pos: usize) {
        let here = self.words.len();
        let offset = (here as isize) - (pos as isize + 2);
        self.words[pos + 1] = offset as i16 as u16;
    }

    fn finish(self) -> Vec<u16> { self.words }
}

// ─── SemOp lowering ──────────────────────────────────────────────────────────

fn lower_op(b: &mut Builder, op: &SemOp) {
    match op {
        SemOp::Set(dest, expr) => {
            lower_expr(b, expr);
            lower_dest(b, dest);
        }

        SemOp::Branch(expr) => {
            lower_expr(b, expr);
            b.emit(Instr::Mask15);
            b.emit(Instr::Store15(addr::Z));
            b.emit(Instr::Ret); // branch terminates this instruction
        }

        SemOp::BranchIf(cond, expr) => {
            lower_cond(b, cond);
            // Jump OVER the branch body if condition is false.
            let skip = b.jump_not_placeholder();
            // Condition true: set Z to target, return early.
            lower_expr(b, expr);
            b.emit(Instr::Mask15);
            b.emit(Instr::Store15(addr::Z));
            b.emit(Instr::Ret);
            b.patch(skip);
        }

        SemOp::SetFlag(Flag::Extend) => {
            b.emit(Instr::PushImm(1));
            b.emit(Instr::Store(addr::EXTEND));
        }
        SemOp::SetFlag(Flag::Inhint) => {
            b.emit(Instr::PushImm(1));
            b.emit(Instr::Store(addr::INHINT));
        }
        SemOp::ClearFlag(Flag::Extend) => {
            b.emit(Instr::PushImm(0));
            b.emit(Instr::Store(addr::EXTEND));
        }
        SemOp::ClearFlag(Flag::Inhint) => {
            b.emit(Instr::PushImm(0));
            b.emit(Instr::Store(addr::INHINT));
        }
    }
}

// ─── Expression lowering ─────────────────────────────────────────────────────

/// Emit bytecode that leaves `eval(expr)` on top of the stack.
fn lower_expr(b: &mut Builder, expr: &Expr) {
    match expr {
        Expr::Lit(n)    => b.emit(Instr::PushImm(*n)),
        Expr::A         => b.emit(Instr::Load(addr::A)),
        Expr::L         => b.emit(Instr::Load(addr::L)),
        Expr::Q         => b.emit(Instr::Load(addr::Q)),
        Expr::Z         => b.emit(Instr::Load(addr::Z)),
        Expr::Tmp       => b.emit(Instr::Load(addr::TMP)),
        Expr::Mem       => b.emit(Instr::LoadOff),
        Expr::MemHi     => b.emit(Instr::LoadOff1),
        Expr::Channel   => b.emit(Instr::LoadChanOff),
        Expr::Operand   => b.emit(Instr::GetOff),

        Expr::PcRel(offset) => {
            // Z + offset where Z is post-fetch PC; TC2 add wraps naturally.
            b.emit(Instr::Load(addr::Z));
            b.emit(Instr::PushImm(*offset as i16 as u16));
            b.emit(Instr::Add);
            b.emit(Instr::Mask15);
        }
        Expr::MemAt(a)  => b.emit(Instr::Load(*a)),
        Expr::InstrWord => b.emit(Instr::Load(addr::INSTR_WORD)),

        Expr::Deref(inner) => {
            lower_expr(b, inner);
            b.emit(Instr::LoadInd);
        }

        // ── Bitwise (directly TC2) ─────────────────────────────────────────

        Expr::And(a, lhs) => {
            lower_expr(b, a);
            lower_expr(b, lhs);
            b.emit(Instr::And);
        }
        Expr::Or(a, lhs) => {
            lower_expr(b, a);
            lower_expr(b, lhs);
            b.emit(Instr::Or);
        }
        Expr::Xor(a, lhs) => {
            lower_expr(b, a);
            lower_expr(b, lhs);
            b.emit(Instr::Xor);
        }
        // NOT in the AGC semantics is always 15-bit (same as OC negation per exec.rs)
        Expr::Not(inner) => {
            lower_expr(b, inner);
            emit_oc_neg(b);
        }

        // ── OC arithmetic (lowered to TC2 sequences) ───────────────────────

        Expr::OcNeg(inner) => {
            lower_expr(b, inner);
            emit_oc_neg(b);
        }
        Expr::OcAdd(a, lhs) => {
            lower_expr(b, a);
            lower_expr(b, lhs);
            emit_oc_add(b);
        }
        Expr::OcSub(a, lhs) => {
            lower_expr(b, a);
            lower_expr(b, lhs);
            emit_oc_sub(b); // = oc_add(a, oc_neg(b))
        }
        Expr::Augment(inner) => {
            lower_expr(b, inner);
            emit_oc_aug(b);
        }
        Expr::Diminish(inner) => {
            lower_expr(b, inner);
            emit_oc_dim(b);
        }
        Expr::Saturate(inner) => {
            lower_expr(b, inner);
            emit_oc_sat(b);
        }

        // ── OC multiply — convert OC→TC2, IMUL, convert TC2→OC ───────────

        Expr::MulHi(a, lhs) => {
            lower_expr(b, a);
            emit_oc_to_tc2(b);
            lower_expr(b, lhs);
            emit_oc_to_tc2(b);
            b.emit(Instr::ImulHi15);
            emit_tc2_to_oc15(b);
        }
        Expr::MulLo(a, lhs) => {
            lower_expr(b, a);
            emit_oc_to_tc2(b);
            lower_expr(b, lhs);
            emit_oc_to_tc2(b);
            b.emit(Instr::ImulLo15);
            emit_tc2_to_oc15(b);
        }

        // ── OC divide — convert OC→TC2, IDIV, convert TC2→OC ─────────────

        Expr::DivQ(a_hi, a_lo, divisor) => {
            lower_expr(b, a_hi);
            emit_oc_to_tc2(b);
            lower_expr(b, a_lo);
            emit_oc_lo_mag(b);    // lower 15-bit unsigned magnitude for IDIV
            lower_expr(b, divisor);
            emit_oc_to_tc2(b);
            b.emit(Instr::IdivQ15);
            emit_tc2_to_oc15(b);
        }
        Expr::DivR(a_hi, a_lo, divisor) => {
            lower_expr(b, a_hi);
            emit_oc_to_tc2(b);
            lower_expr(b, a_lo);
            emit_oc_lo_mag(b);
            lower_expr(b, divisor);
            emit_oc_to_tc2(b);
            b.emit(Instr::IdivR15);
            emit_tc2_to_oc15(b);
        }

        // ── Double-precision add high ─────────────────────────────────────
        // dp_add_hi(a_hi, a_lo, b_hi, b_lo):
        //   (_, carry) = oc_add(a_lo, b_lo)
        //   hi_sum = oc_add(oc_add(a_hi, b_hi), carry_word)

        Expr::DpAddHi(a_hi, a_lo, b_hi, b_lo) => {
            // Compute carry from lo addition.
            lower_expr(b, a_lo);
            lower_expr(b, b_lo);
            emit_oc_add_with_carry(b); // leaves [carry_word (0 or 1)] on stack

            // Compute oc_add(a_hi, b_hi).
            lower_expr(b, a_hi);
            lower_expr(b, b_hi);
            emit_oc_add(b); // leaves [oc_add(a_hi, b_hi)]

            // Stack is now [carry_word, hi_sum]; add carry to hi_sum.
            b.emit(Instr::Swap);
            // Stack: [hi_sum, carry_word]
            emit_oc_add(b); // oc_add(hi_sum, carry_word)
        }
    }
}

// ─── Destination lowering ─────────────────────────────────────────────────────

/// Emit bytecode that pops the top of stack and stores it to `dest`.
fn lower_dest(b: &mut Builder, dest: &Dest) {
    match dest {
        Dest::A       => b.emit(Instr::Store(addr::A)),   // A keeps overflow (bit 15)
        Dest::L       => b.emit(Instr::Store15(addr::L)),
        Dest::Q       => b.emit(Instr::Store15(addr::Q)),
        Dest::Z       => b.emit(Instr::Store15(addr::Z)),
        Dest::Tmp     => b.emit(Instr::Store(addr::TMP)), // tmp keeps overflow
        Dest::Mem     => b.emit(Instr::StoreOff),
        Dest::MemHi   => b.emit(Instr::StoreOff1),
        Dest::Channel => b.emit(Instr::StoreChanOff),
        Dest::MemAt(a) => b.emit(Instr::Store15(*a)),
        Dest::Deref(inner) => {
            // val is on top; push addr below, then StoreInd needs [..., val, addr].
            lower_expr(b, inner); // push addr (on top now)
            // Stack: [..., val, addr]  — StoreInd pops addr then val.
            b.emit(Instr::StoreInd);
        }
    }
}

// ─── Condition lowering ───────────────────────────────────────────────────────

/// Emit bytecode that leaves a boolean (0 or 1) on the stack.
fn lower_cond(b: &mut Builder, cond: &Cond) {
    match cond {
        Cond::IsPositive(e) => {
            lower_expr(b, e);
            b.emit(Instr::Mask15);
            b.emit(Instr::IsPos);
        }
        Cond::IsPlusZero(e) => {
            lower_expr(b, e);
            b.emit(Instr::Mask15);
            b.emit(Instr::IsPlusZero);
        }
        Cond::IsNegative(e) => {
            lower_expr(b, e);
            b.emit(Instr::Mask15);
            b.emit(Instr::IsNeg);
        }
        Cond::IsMinusZero(e) => {
            lower_expr(b, e);
            b.emit(Instr::Mask15);
            b.emit(Instr::IsMinusZero);
        }
        Cond::IsZeroOrNeg(e) => {
            lower_expr(b, e);
            b.emit(Instr::Mask15);
            b.emit(Instr::IsZeroOrNeg);
        }
        Cond::HasOverflow(e) => {
            lower_expr(b, e);
            b.emit(Instr::HasOverflow); // takes full 16-bit value
        }
        Cond::And(a, lhs) => {
            lower_cond(b, a);
            lower_cond(b, lhs);
            b.emit(Instr::BoolAnd);
        }
        Cond::Not(inner) => {
            lower_cond(b, inner);
            b.emit(Instr::BoolNot);
        }
    }
}

// ─── OC arithmetic as TC2 sequences ──────────────────────────────────────────

// Convention: all helpers document stack state as [..., bottom, top]
// where the rightmost element is the top.

/// `oc_neg`: stack [..., x] → [..., (!x) & 0x7FFF]  (2 words)
#[inline]
fn emit_oc_neg(b: &mut Builder) {
    b.emit(Instr::Not);    // !x  (16-bit)
    b.emit(Instr::Mask15); // & 0x7FFF
}

/// `oc_add`: stack [..., a, b] → [..., result]
///
/// TC2 lowering:
/// ```text
/// b_m  = b & 0x7FFF
/// a_m  = a & 0x7FFF
/// raw  = a_m + b_m          (TC2 add; range 0..0xFFFE)
/// carry= raw >> 15           (0 or 1)
/// result = (raw + carry) & 0x7FFF
/// ```
fn emit_oc_add(b: &mut Builder) {
    b.emit(Instr::Mask15);       // [a, b_m]
    b.emit(Instr::Swap);         // [b_m, a]
    b.emit(Instr::Mask15);       // [b_m, a_m]
    b.emit(Instr::Swap);         // [a_m, b_m]
    b.emit(Instr::Add);          // [raw]
    b.emit(Instr::Dup);          // [raw, raw]
    b.emit(Instr::Lshr(15));     // [raw, carry]
    b.emit(Instr::Add);          // [raw + carry]
    b.emit(Instr::Mask15);       // [result]
}

/// `oc_add` returning the carry word (0 or 1) for `dp_add_hi`.
/// stack [..., a, b] → [..., carry]  (carry = 1 if oc_add overflowed, else 0)
fn emit_oc_add_with_carry(b: &mut Builder) {
    b.emit(Instr::Mask15);
    b.emit(Instr::Swap);
    b.emit(Instr::Mask15);
    b.emit(Instr::Swap);
    b.emit(Instr::Add);          // [raw]
    b.emit(Instr::Lshr(15));     // [carry]   raw >> 15
}

/// `oc_sub`: stack [..., a, b] → [..., oc_sub(a, b)]
fn emit_oc_sub(b: &mut Builder) {
    // oc_sub(a, b) = oc_add(a, oc_neg(b))
    b.emit(Instr::Not);    // [a, !b]
    b.emit(Instr::Mask15); // [a, oc_neg_b]
    emit_oc_add(b);
}

/// `oc_sat`: stack [..., a_16bit] → [..., result_15bit]
///
/// Saturates 16-bit accumulator value (may have overflow in bit 15) to ±max.
/// - No overflow (bits 14 and 15 equal): strip bit 15.
/// - Positive overflow (bit15=0, bit14=1): return `0x3FFF`.
/// - Negative overflow (bit15=1, bit14=0): return `0x4000`.
fn emit_oc_sat(b: &mut Builder) {
    // Compute has_overflow = bit14 XOR bit15.
    // Trick: (a ^ (a >> 1)) >> 14 & 1 gives bit14 XOR bit15.
    b.emit(Instr::Dup);         // [a, a]
    b.emit(Instr::Dup);         // [a, a, a]
    b.emit(Instr::Lshr(1));     // [a, a, a>>1]
    b.emit(Instr::Xor);         // [a, a ^ (a>>1)]
    b.emit(Instr::Lshr(14));    // [a, (a^(a>>1))>>14]
    b.emit(Instr::PushImm(1));  // [a, .., 1]
    b.emit(Instr::And);         // [a, has_ov]  (0 or 1)

    let j_no_ov = b.jump_not_placeholder(); // JUMP_NOT → no_overflow

    // ── Overflow case: [a]  (has_ov consumed)
    b.emit(Instr::Dup);         // [a, a]
    b.emit(Instr::Lshr(15));    // [a, bit15]  (0=pos_ov, 1=neg_ov)
    let j_neg_ov = b.jump_if_placeholder();  // JUMP_IF → neg_overflow

    // Positive overflow: [a]
    b.emit(Instr::Drop);
    b.emit(Instr::PushImm(0x3FFF));
    let j_end1 = b.jump_placeholder();

    // neg_overflow: [a]
    b.patch(j_neg_ov);
    b.emit(Instr::Drop);
    b.emit(Instr::PushImm(0x4000));
    let j_end2 = b.jump_placeholder();

    // no_overflow: [a]
    b.patch(j_no_ov);
    b.emit(Instr::Mask15);

    b.patch(j_end1);
    b.patch(j_end2);
}

/// `oc_aug`: stack [..., w] → [..., oc_augment(w)]
///
/// Move w one step away from zero.
/// - If w is OC zero (0x0000 or 0x7FFF): return unchanged.
/// - If positive: add +1 in OC  (= `oc_add(w, 1)`).
/// - If negative: add −1 in OC  (= `oc_add(w, 0x7FFE)`).
fn emit_oc_aug(b: &mut Builder) {
    b.emit(Instr::Mask15);        // [w_m]

    // Check for OC zero (plus-zero or minus-zero).
    b.emit(Instr::Dup);           // [w_m, w_m]
    b.emit(Instr::Dup);           // [w_m, w_m, w_m]
    b.emit(Instr::IsPlusZero);    // [w_m, w_m, is_pz]
    b.emit(Instr::Swap);          // [w_m, is_pz, w_m]
    b.emit(Instr::IsMinusZero);   // [w_m, is_pz, is_mz]
    b.emit(Instr::Or);            // [w_m, is_zero]
    let j_zero = b.jump_if_placeholder(); // JUMP_IF → zero_end (return w_m unchanged)

    // Not zero: [w_m]
    b.emit(Instr::Dup);           // [w_m, w_m]
    b.emit(Instr::Lshr(14));      // [w_m, sign]  (0=positive, 1=negative)
    let j_neg = b.jump_if_placeholder();  // JUMP_IF → negative

    // Positive: oc_add(w_m, 1)
    b.emit(Instr::PushImm(1));    // [w_m, 1]
    emit_oc_add(b);               // [result]
    let j_end = b.jump_placeholder();

    // Negative: oc_add(w_m, 0x7FFE)  (0x7FFE = -1 in OC)
    b.patch(j_neg);
    b.emit(Instr::PushImm(0x7FFE)); // [w_m, 0x7FFE]
    emit_oc_add(b);                 // [result]
    // fall through to end

    b.patch(j_zero);
    b.patch(j_end);
}

/// `oc_dim`: stack [..., w] → [..., oc_diminish(w)]
///
/// Move w one step toward zero.
/// - If w is OC zero: return unchanged.
/// - If positive: if w==1 return +0 (0x0000), else return w−1.
/// - If negative: magnitude = (~w) & 0x3FFF; if magnitude==1 return −0 (0x7FFF),
///   else return ~((magnitude−1) & 0x3FFF) & 0x7FFF.
fn emit_oc_dim(b: &mut Builder) {
    b.emit(Instr::Mask15);       // [w_m]

    // Check for OC zero.
    b.emit(Instr::Dup);          // [w_m, w_m]
    b.emit(Instr::Dup);          // [w_m, w_m, w_m]
    b.emit(Instr::IsPlusZero);   // [w_m, w_m, is_pz]
    b.emit(Instr::Swap);         // [w_m, is_pz, w_m]
    b.emit(Instr::IsMinusZero);  // [w_m, is_pz, is_mz]
    b.emit(Instr::Or);           // [w_m, is_zero]
    let j_zero = b.jump_if_placeholder(); // JUMP_IF → zero_end

    // Not zero: [w_m]
    b.emit(Instr::Dup);          // [w_m, w_m]
    b.emit(Instr::Lshr(14));     // [w_m, sign]
    let j_neg = b.jump_if_placeholder();  // JUMP_IF → negative

    // Positive: check if w_m == 1.
    // w_m == 1  ↔  IS_PLUS_ZERO(w_m − 1)
    b.emit(Instr::Dup);          // [w_m, w_m]
    b.emit(Instr::PushImm(1));   // [w_m, w_m, 1]
    b.emit(Instr::Sub);          // [w_m, w_m − 1]
    b.emit(Instr::IsPlusZero);   // [w_m, (w_m == 1)]
    let j_pos_one = b.jump_if_placeholder(); // JUMP_IF → pos_is_one
    // w_m != 1: return w_m − 1
    b.emit(Instr::PushImm(1));
    b.emit(Instr::Sub);          // [w_m − 1]
    let j_end1 = b.jump_placeholder();
    // pos_is_one: return plus-zero (0x0000)
    b.patch(j_pos_one);
    b.emit(Instr::Drop);
    b.emit(Instr::PushImm(0x0000));
    let j_end2 = b.jump_placeholder();

    // Negative: [w_m]
    b.patch(j_neg);
    b.emit(Instr::Not);          // [~w_m]
    b.emit(Instr::PushImm(0x3FFF));
    b.emit(Instr::And);          // [mag = (~w_m) & 0x3FFF]
    // Check if mag == 1.
    b.emit(Instr::Dup);          // [mag, mag]
    b.emit(Instr::PushImm(1));
    b.emit(Instr::Sub);          // [mag, mag − 1]
    b.emit(Instr::IsPlusZero);   // [mag, (mag == 1)]
    let j_neg_one = b.jump_if_placeholder(); // JUMP_IF → neg_is_one
    // mag != 1: return ~((mag−1) & 0x3FFF) & 0x7FFF
    b.emit(Instr::PushImm(1));
    b.emit(Instr::Sub);          // [mag − 1]
    b.emit(Instr::PushImm(0x3FFF));
    b.emit(Instr::And);          // [(mag−1) & 0x3FFF]
    b.emit(Instr::Not);          // [~((mag−1) & 0x3FFF)]
    b.emit(Instr::Mask15);       // [result]
    let j_end3 = b.jump_placeholder();
    // neg_is_one: return minus-zero (0x7FFF)
    b.patch(j_neg_one);
    b.emit(Instr::Drop);
    b.emit(Instr::PushImm(0x7FFF));

    b.patch(j_zero);
    b.patch(j_end1);
    b.patch(j_end2);
    b.patch(j_end3);
}

// ─── OC ↔ TC2 signed conversions (for mul/div lowering) ──────────────────────

/// Convert top-of-stack from OC 15-bit to TC2 signed 16-bit.
///
/// ```text
/// 0x0000 → 0         (plus-zero → TC2 zero)
/// 0x7FFF → 0         (minus-zero → TC2 zero)
/// 0x0001..0x3FFF → same (positive OC = positive TC2)
/// 0x4000..0x7FFE → -(magnitude) in TC2  where magnitude = (~oc) & 0x3FFF
/// ```
fn emit_oc_to_tc2(b: &mut Builder) {
    b.emit(Instr::Mask15);           // [v]  ensure 15-bit

    // Minus-zero → TC2 zero.
    b.emit(Instr::Dup);              // [v, v]
    b.emit(Instr::IsMinusZero);      // [v, is_mz]
    let j_not_mz = b.jump_not_placeholder(); // JUMP_NOT → not_minus_zero
    // minus-zero: replace with 0
    b.emit(Instr::Drop);
    b.emit(Instr::PushImm(0));
    let j_end1 = b.jump_placeholder();

    // not_minus_zero: [v]
    b.patch(j_not_mz);
    b.emit(Instr::Dup);              // [v, v]
    b.emit(Instr::Lshr(14));         // [v, bit14]  OC sign bit
    let j_neg = b.jump_if_placeholder(); // JUMP_IF → negative

    // Positive: return as-is.
    let j_end2 = b.jump_placeholder();

    // Negative: magnitude = (~v) & 0x3FFF; TC2 = -magnitude = ~magnitude + 1.
    b.patch(j_neg);
    b.emit(Instr::Not);              // [~v]
    b.emit(Instr::PushImm(0x3FFF));
    b.emit(Instr::And);              // [mag = (~v) & 0x3FFF]
    b.emit(Instr::Not);              // [~mag]
    b.emit(Instr::PushImm(1));
    b.emit(Instr::Add);              // [-mag = TC2 negative]

    b.patch(j_end1);
    b.patch(j_end2);
}

/// Extract unsigned 15-bit magnitude from OC for use as `a_lo` in `IDIV_Q15`/`IDIV_R15`.
///
/// The VM's IDIV ops take `a_lo` as an unsigned 15-bit magnitude (absolute value),
/// matching the convention in `oc_div` where `a_lo` magnitude is `|oc15_to_i32(a_lo)|`.
fn emit_oc_lo_mag(b: &mut Builder) {
    b.emit(Instr::Mask15);       // [v]  ensure 15-bit
    // Check if negative; if so, compute magnitude = (~v) & 0x3FFF.
    b.emit(Instr::Dup);          // [v, v]
    b.emit(Instr::Lshr(14));     // [v, bit14]
    let j_neg = b.jump_if_placeholder();
    // Positive (or zero): magnitude is the value itself.
    let j_end = b.jump_placeholder();
    // Negative:
    b.patch(j_neg);
    b.emit(Instr::Not);
    b.emit(Instr::PushImm(0x3FFF));
    b.emit(Instr::And);          // magnitude
    b.patch(j_end);
}

/// Convert TC2 signed 16-bit on top-of-stack to OC 15-bit.
///
/// ```text
///  0       → 0x0000  (plus-zero)
///  positive → same value (fits in 14 bits for OC purposes)
///  negative → (~magnitude) & 0x7FFF  where magnitude = -v
/// ```
fn emit_tc2_to_oc15(b: &mut Builder) {
    b.emit(Instr::Dup);          // [v, v]
    b.emit(Instr::Lshr(15));     // [v, bit15]  TC2 sign
    let j_neg = b.jump_if_placeholder(); // JUMP_IF → negative

    // Non-negative: mask to 15 bits.
    b.emit(Instr::Mask15);
    let j_end = b.jump_placeholder();

    // Negative: OC = (~magnitude) & 0x7FFF  where magnitude = -v
    b.patch(j_neg);
    b.emit(Instr::Neg);          // [-v = magnitude]
    b.emit(Instr::Not);          // [~magnitude]
    b.emit(Instr::Mask15);       // [(!magnitude) & 0x7FFF]

    b.patch(j_end);
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{BytecodeVm, addr};
    use agc_isa::parse_sem;

    fn run(vm: &mut BytecodeVm, ops_text: &str, operand: u16) {
        let ops = parse_sem(ops_text).expect("parse_sem");
        let code = lower_sem(operand, &ops);
        vm.exec(&code).expect("exec");
    }

    // ── TC2 lowering of OC arithmetic ─────────────────────────────────────

    #[test]
    fn oc_neg_plus_one() {
        // OC negation of 1 = 0x7FFE (which represents -1)
        let mut b = Builder::new();
        Instr::PushImm(1).encode(&mut b.words);
        emit_oc_neg(&mut b);
        Instr::Ret.encode(&mut b.words);
        // The result ends up on the stack; store to a known address to read it.
        let _code = b.finish();
        // Re-emit with a store
        let mut b2 = Builder::new();
        Instr::PushImm(1).encode(&mut b2.words);
        emit_oc_neg(&mut b2);
        Instr::Store(0x0100).encode(&mut b2.words);
        Instr::Ret.encode(&mut b2.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b2.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x7FFE, "oc_neg(1) = 0x7FFE");
    }

    #[test]
    fn oc_add_simple() {
        let mut b = Builder::new();
        Instr::PushImm(3).encode(&mut b.words);  // a
        Instr::PushImm(4).encode(&mut b.words);  // b
        emit_oc_add(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 7, "oc_add(3, 4) = 7");
    }

    #[test]
    fn oc_add_end_around_carry() {
        // 0x7FFE + 0x0002: raw = 0x8000, carry = 1, result = (0x8000+1)&0x7FFF = 0x0001
        let mut b = Builder::new();
        Instr::PushImm(0x7FFE).encode(&mut b.words);
        Instr::PushImm(0x0002).encode(&mut b.words);
        emit_oc_add(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x0001, "end-around carry");
    }

    #[test]
    fn oc_sub_simple() {
        // oc_sub(5, 3) = oc_add(5, oc_neg(3)) = oc_add(5, 0x7FFC) → 2
        let mut b = Builder::new();
        Instr::PushImm(5).encode(&mut b.words);
        Instr::PushImm(3).encode(&mut b.words);
        emit_oc_sub(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 2, "oc_sub(5, 3) = 2");
    }

    #[test]
    fn oc_sat_no_overflow() {
        // Normal value: no overflow → strip bit 15.
        let mut b = Builder::new();
        Instr::PushImm(42).encode(&mut b.words);
        emit_oc_sat(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 42);
    }

    #[test]
    fn oc_sat_positive_overflow() {
        // bit15=0, bit14=1 → positive overflow → 0x3FFF
        let mut b = Builder::new();
        Instr::PushImm(0x4001).encode(&mut b.words); // bit14=1, bit15=0
        emit_oc_sat(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x3FFF);
    }

    #[test]
    fn oc_sat_negative_overflow() {
        // bit15=1, bit14=0 → negative overflow → 0x4000
        let mut b = Builder::new();
        Instr::PushImm(0x8001).encode(&mut b.words); // bit15=1, bit14=0
        emit_oc_sat(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x4000);
    }

    #[test]
    fn oc_aug_positive() {
        let mut b = Builder::new();
        Instr::PushImm(5).encode(&mut b.words);
        emit_oc_aug(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 6, "aug(5) = 6");
    }

    #[test]
    fn oc_aug_negative() {
        // Negative -2 in OC = 0x7FFD; augment → -3 = 0x7FFC
        let mut b = Builder::new();
        Instr::PushImm(0x7FFD).encode(&mut b.words);
        emit_oc_aug(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x7FFC, "aug(-2) = -3");
    }

    #[test]
    fn oc_aug_plus_zero_unchanged() {
        let mut b = Builder::new();
        Instr::PushImm(0x0000).encode(&mut b.words);
        emit_oc_aug(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x0000, "aug(+0) = +0");
    }

    #[test]
    fn oc_dim_positive() {
        let mut b = Builder::new();
        Instr::PushImm(5).encode(&mut b.words);
        emit_oc_dim(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 4, "dim(5) = 4");
    }

    #[test]
    fn oc_dim_pos_one_to_plus_zero() {
        let mut b = Builder::new();
        Instr::PushImm(1).encode(&mut b.words);
        emit_oc_dim(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x0000, "dim(1) = +0");
    }

    #[test]
    fn oc_dim_neg_one_to_minus_zero() {
        // -1 in OC = 0x7FFE; dim(-1) = -0 = 0x7FFF
        let mut b = Builder::new();
        Instr::PushImm(0x7FFE).encode(&mut b.words);
        emit_oc_dim(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x7FFF, "dim(-1) = -0");
    }

    #[test]
    fn oc_dim_minus_zero_unchanged() {
        let mut b = Builder::new();
        Instr::PushImm(0x7FFF).encode(&mut b.words);
        emit_oc_dim(&mut b);
        Instr::Store(0x0100).encode(&mut b.words);
        Instr::Ret.encode(&mut b.words);
        let mut vm = BytecodeVm::new();
        vm.exec(&b.finish()).unwrap();
        assert_eq!(vm.mem_load(0x0100), 0x7FFF, "dim(-0) = -0");
    }

    // ── SemOp-level tests (lowering paths) ────────────────────────────────

    #[test]
    fn ca_loads_mem_to_a() {
        let mut vm = BytecodeVm::new();
        vm.mem_store(0o100, 42);
        run(&mut vm, "set A mem", 0o100);
        assert_eq!(vm.mem_load(addr::A), 42);
    }

    #[test]
    fn ad_oc_add() {
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::A, 10);
        vm.mem_store(0o100, 5);
        run(&mut vm, "set A oc_add(A,mem)", 0o100);
        assert_eq!(vm.mem_load(addr::A), 15);
    }

    #[test]
    fn branch_if_not_taken() {
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::A, 1); // positive — condition false
        vm.mem_store(addr::Z, 0o200);
        run(&mut vm, "branch_if is_plus_zero(A) operand", 0o500);
        assert_eq!(vm.mem_load(addr::Z), 0o200, "branch not taken");
    }

    #[test]
    fn branch_if_taken() {
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::A, 0); // plus-zero — condition true
        run(&mut vm, "branch_if is_plus_zero(A) operand", 0o500);
        assert_eq!(vm.mem_load(addr::Z), 0o500, "branch taken");
    }

    #[test]
    fn set_flag_extend() {
        let mut vm = BytecodeVm::new();
        run(&mut vm, "set_flag extend", 0);
        assert_eq!(vm.mem_load(addr::EXTEND), 1);
    }

    #[test]
    fn clear_flag_extend() {
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::EXTEND, 1);
        run(&mut vm, "clear_flag extend", 0);
        assert_eq!(vm.mem_load(addr::EXTEND), 0);
    }

    #[test]
    fn xch_swap() {
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::A, 0o777);
        vm.mem_store(0o100, 0o123);
        run(&mut vm, "set tmp mem\nset mem A\nset A tmp", 0o100);
        assert_eq!(vm.mem_load(addr::A), 0o123);
        assert_eq!(vm.mem_load(0o100), 0o777);
    }

    #[test]
    fn mul_hi_lo() {
        // 3 × 4 = 12; fits in lo, hi = 0
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::A, 3);
        vm.mem_store(0o100, 4);
        run(&mut vm, "set L mul_lo(A,mem)\nset A mul_hi(A,mem)", 0o100);
        assert_eq!(vm.mem_load(addr::L), 12);
        assert_eq!(vm.mem_load(addr::A), 0);
    }

    #[test]
    fn div_simple() {
        // DV: set tmp A; set A div_q(tmp,L,mem); set L div_r(tmp,L,mem)
        // 10 / 3 = quotient 3, remainder 1
        let mut vm = BytecodeVm::new();
        vm.mem_store(addr::A, 0);  // a_hi
        vm.mem_store(addr::L, 10); // a_lo
        vm.mem_store(0o100, 3);    // divisor
        run(&mut vm, "set tmp A\nset A div_q(tmp,L,mem)\nset L div_r(tmp,L,mem)", 0o100);
        assert_eq!(vm.mem_load(addr::A), 3);
        assert_eq!(vm.mem_load(addr::L), 1);
    }
}
