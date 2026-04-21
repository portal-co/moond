//! TC2 16-bit stack bytecode: opcodes, [`Instr`] enum, encode/decode.
//!
//! The VM is purely TC2 (two's-complement). No ones-complement arithmetic
//! opcodes exist here; OC operations are lowered to sequences of TC2
//! instructions in [`crate::lower`].
//!
//! # Encoding
//!
//! - Simple opcodes (`0x0001`–`0x002F`): 1 word.
//! - Two-word opcodes (`0x0100`–`0x010A`): opcode word + u16 operand word.
//! - Three-word opcodes (`0x0200`–`0x02FF`): opcode word + two u16 operand words.
//!
//! Jump offsets are signed (stored as `u16` with TC2 interpretation).
//! Offset 0 means "next instruction after this one" (jump over nothing).
//! The offset is relative to the word *after* the two-word instruction.

// ─── Opcode constants ─────────────────────────────────────────────────────────

/// Raw opcode values for the TC2 VM.
pub mod op {
    // ── TC2 arithmetic (1-word) ────────────────────────────────────────────

    /// End execution and return to caller.
    pub const RET:            u16 = 0x0001;
    /// TC2 add:             pop b, pop a, push `a.wrapping_add(b)`.
    pub const ADD:            u16 = 0x0002;
    /// TC2 subtract:        pop b, pop a, push `a.wrapping_sub(b)`.
    pub const SUB:            u16 = 0x0003;
    /// Bitwise AND:         pop b, pop a, push `a & b`.
    pub const AND:            u16 = 0x0004;
    /// Bitwise OR:          pop b, pop a, push `a | b`.
    pub const OR:             u16 = 0x0005;
    /// Bitwise XOR:         pop b, pop a, push `a ^ b`.
    pub const XOR:            u16 = 0x0006;
    /// 16-bit bitwise NOT:  pop x, push `!x`.
    pub const NOT:            u16 = 0x0007;
    /// 15-bit mask:         pop x, push `x & 0x7FFF`.
    pub const MASK15:         u16 = 0x0008;
    /// TC2 negate:          pop x, push `x.wrapping_neg()` (= `(!x).wrapping_add(1)`).
    pub const NEG:            u16 = 0x0009;
    /// Logical right shift: pop amount (u16), pop x, push `x >> amount`.
    pub const LSHR_STK:       u16 = 0x000A;
    /// Logical left shift:  pop amount (u16), pop x, push `(x << amount) & 0xFFFF`.
    pub const LSHL_STK:       u16 = 0x000B;

    // ── TC2 wide integer ops (signed, for OC mul/div lowering) ────────────

    /// Signed 16×16→32 multiply, high word:
    /// pop b (as i16), pop a (as i16), push upper 15 bits of `(a*b)` (i.e. `(a*b) >> 15`).
    pub const IMUL_HI15:      u16 = 0x000C;
    /// Signed 16×16→32 multiply, low word:
    /// pop b, pop a, push lower 15 bits of `(a*b)` (i.e. `(a*b) & 0x7FFF`).
    pub const IMUL_LO15:      u16 = 0x000D;
    /// Signed divide quotient (for OC div lowering):
    /// pop divisor (as i32 from u16 sign-ext), pop a_lo (15-bit magnitude u16),
    /// pop a_hi (as i16), push TC2 signed quotient.
    pub const IDIV_Q15:       u16 = 0x000E;
    /// Signed divide remainder: same stack, push TC2 signed remainder.
    pub const IDIV_R15:       u16 = 0x000F;

    // ── OC bit-pattern predicates (push 1 if true, 0 if false) ───────────
    // These check bit patterns only — no OC arithmetic.

    /// OC positive: pop x (15-bit), push `1` if `is_positive(x)` else `0`.
    pub const IS_POS:         u16 = 0x0010;
    /// OC plus-zero: pop x (15-bit), push `1` if `x & 0x7FFF == 0` else `0`.
    pub const IS_PLUS_ZERO:   u16 = 0x0011;
    /// OC negative: pop x (15-bit), push `1` if `(x & 0x4000) != 0 && x != 0x7FFF`.
    pub const IS_NEG:         u16 = 0x0012;
    /// OC minus-zero: pop x (15-bit), push `1` if `x & 0x7FFF == 0x7FFF`.
    pub const IS_MINUS_ZERO:  u16 = 0x0013;
    /// OC zero-or-negative: pop x (15-bit), push `1` if zero or negative in OC.
    pub const IS_ZERO_OR_NEG: u16 = 0x0014;
    /// TC2 overflow (for TS): pop x (16-bit), push `1` if bits 14 and 15 differ.
    pub const HAS_OVERFLOW:   u16 = 0x0015;
    /// Boolean AND: pop b (0/1), pop a (0/1), push `a & b`.
    pub const BOOL_AND:       u16 = 0x0016;
    /// Boolean NOT: pop x, push `1` if `x == 0` else `0`.
    pub const BOOL_NOT:       u16 = 0x0017;

    // ── Stack manipulation ─────────────────────────────────────────────────

    /// Duplicate top of stack.
    pub const DUP:            u16 = 0x0018;
    /// Swap top two values.
    pub const SWAP:           u16 = 0x0019;
    /// Discard top of stack.
    pub const DROP:           u16 = 0x001A;

    // ── T register ────────────────────────────────────────────────────────

    /// Push T onto the stack.
    pub const LOAD_T:         u16 = 0x001B;
    /// Pop stack to T.
    pub const STORE_T:        u16 = 0x001C;

    // ── OFF-relative memory ────────────────────────────────────────────────

    /// Push `mem[OFF]`.
    pub const LOAD_OFF:       u16 = 0x001D;
    /// Pop and store to `mem[OFF]` with 15-bit mask.
    pub const STORE_OFF:      u16 = 0x001E;
    /// Push `mem[OFF + 1]`.
    pub const LOAD_OFF1:      u16 = 0x001F;
    /// Pop and store to `mem[OFF + 1]` with 15-bit mask.
    pub const STORE_OFF1:     u16 = 0x0020;
    /// Push value of OFF.
    pub const GET_OFF:        u16 = 0x0021;
    /// Pop stack to OFF.
    pub const SET_OFF_STACK:  u16 = 0x0022;

    // ── Channel / indirect ─────────────────────────────────────────────────

    /// Push `channel[OFF]`.
    pub const LOAD_CHAN_OFF:  u16 = 0x0023;
    /// Pop and store to `channel[OFF]` with 15-bit mask.
    pub const STORE_CHAN_OFF: u16 = 0x0024;
    /// Indirect load: pop addr, push `mem[addr]`.
    pub const LOAD_IND:       u16 = 0x0025;
    /// Indirect store: pop addr, pop val, store `val & 0x7FFF` to `mem[addr]`.
    pub const STORE_IND:      u16 = 0x0026;

    // ── Two-word opcodes (opcode + u16 operand) ───────────────────────────

    /// Push immediate u16.
    pub const PUSH_IMM:       u16 = 0x0100;
    /// Load from absolute address: push `mem[addr]`.
    pub const LOAD:           u16 = 0x0101;
    /// Store to absolute address (no mask): `mem[addr]` ← pop.
    pub const STORE:          u16 = 0x0102;
    /// Store with 15-bit mask: `mem[addr]` ← `pop & 0x7FFF`.
    pub const STORE15:        u16 = 0x0103;
    /// Unconditional jump. Operand: i16 offset from end of this instruction.
    pub const JUMP:           u16 = 0x0104;
    /// Jump if non-zero: pop cond, jump if `cond != 0`. Operand: i16 offset.
    pub const JUMP_IF:        u16 = 0x0105;
    /// Jump if zero: pop cond, jump if `cond == 0`. Operand: i16 offset.
    pub const JUMP_NOT:       u16 = 0x0106;
    /// Set OFF to immediate value.
    pub const SET_OFF:        u16 = 0x0107;
    /// Logical right shift by immediate: pop x, push `x >> k`.
    pub const LSHR:           u16 = 0x0108;
    /// Logical left shift by immediate: pop x, push `(x << k) & 0xFFFF`.
    pub const LSHL:           u16 = 0x0109;

    // ── Three-word opcodes (opcode + operand-0 + operand-1) ───────────────

    /// Call a host (WASM import / C extern) function by slot index.
    ///
    /// Encoding: `[HOST_CALL, slot:u16, pack(n_args:u8, n_results:u8):u16]`
    ///
    /// * `slot` — index into the backend-provided host-function table.
    /// * `n_args` — number of i32 values popped from the TC2 stack and passed
    ///   as arguments to the host function.  Stack depth must be ≥ n_args.
    /// * `n_results` — number of i32 values pushed onto the TC2 stack after
    ///   the call returns (0 or 1).
    ///
    /// In the WASM backend, slot `s` maps to WASM function index
    /// `NUM_STANDARD_IMPORTS + s`.  In the C backend it maps to a
    /// user-defined `AGC_HOST_CALL` macro.
    pub const HOST_CALL:      u16 = 0x0200;
}

// ─── Instruction enum ─────────────────────────────────────────────────────────

/// Decoded TC2 VM instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    // 1-word
    Ret,
    Add, Sub, And, Or, Xor, Not, Mask15, Neg,
    LshrStk, LshlStk,
    ImulHi15, ImulLo15, IdivQ15, IdivR15,
    IsPos, IsPlusZero, IsNeg, IsMinusZero, IsZeroOrNeg, HasOverflow,
    BoolAnd, BoolNot,
    Dup, Swap, Drop,
    LoadT, StoreT,
    LoadOff, StoreOff, LoadOff1, StoreOff1, GetOff, SetOffStack,
    LoadChanOff, StoreChanOff, LoadInd, StoreInd,
    // 2-word
    PushImm(u16),
    Load(u16),
    Store(u16),
    Store15(u16),
    Jump(i16),
    JumpIf(i16),
    JumpNot(i16),
    SetOff(u16),
    Lshr(u16),
    Lshl(u16),
    /// Call host function at `slot`, passing `n_args` stack values; push `n_results` (0 or 1).
    HostCall { slot: u16, n_args: u8, n_results: u8 },
}

impl Instr {
    /// Number of u16 words this instruction encodes to.
    pub fn word_len(&self) -> usize {
        match self {
            Instr::PushImm(_) | Instr::Load(_) | Instr::Store(_) | Instr::Store15(_)
            | Instr::Jump(_) | Instr::JumpIf(_) | Instr::JumpNot(_) | Instr::SetOff(_)
            | Instr::Lshr(_) | Instr::Lshl(_) => 2,
            Instr::HostCall { .. } => 3,
            _ => 1,
        }
    }

    /// Append the encoded words to `out`.
    pub fn encode(&self, out: &mut Vec<u16>) {
        use op::*;
        match self {
            Instr::Ret          => out.push(RET),
            Instr::Add          => out.push(ADD),
            Instr::Sub          => out.push(SUB),
            Instr::And          => out.push(AND),
            Instr::Or           => out.push(OR),
            Instr::Xor          => out.push(XOR),
            Instr::Not          => out.push(NOT),
            Instr::Mask15       => out.push(MASK15),
            Instr::Neg          => out.push(NEG),
            Instr::LshrStk      => out.push(LSHR_STK),
            Instr::LshlStk      => out.push(LSHL_STK),
            Instr::ImulHi15     => out.push(IMUL_HI15),
            Instr::ImulLo15     => out.push(IMUL_LO15),
            Instr::IdivQ15      => out.push(IDIV_Q15),
            Instr::IdivR15      => out.push(IDIV_R15),
            Instr::IsPos        => out.push(IS_POS),
            Instr::IsPlusZero   => out.push(IS_PLUS_ZERO),
            Instr::IsNeg        => out.push(IS_NEG),
            Instr::IsMinusZero  => out.push(IS_MINUS_ZERO),
            Instr::IsZeroOrNeg  => out.push(IS_ZERO_OR_NEG),
            Instr::HasOverflow  => out.push(HAS_OVERFLOW),
            Instr::BoolAnd      => out.push(BOOL_AND),
            Instr::BoolNot      => out.push(BOOL_NOT),
            Instr::Dup          => out.push(DUP),
            Instr::Swap         => out.push(SWAP),
            Instr::Drop         => out.push(DROP),
            Instr::LoadT        => out.push(LOAD_T),
            Instr::StoreT       => out.push(STORE_T),
            Instr::LoadOff      => out.push(LOAD_OFF),
            Instr::StoreOff     => out.push(STORE_OFF),
            Instr::LoadOff1     => out.push(LOAD_OFF1),
            Instr::StoreOff1    => out.push(STORE_OFF1),
            Instr::GetOff       => out.push(GET_OFF),
            Instr::SetOffStack  => out.push(SET_OFF_STACK),
            Instr::LoadChanOff  => out.push(LOAD_CHAN_OFF),
            Instr::StoreChanOff => out.push(STORE_CHAN_OFF),
            Instr::LoadInd      => out.push(LOAD_IND),
            Instr::StoreInd     => out.push(STORE_IND),
            Instr::PushImm(v)   => { out.push(PUSH_IMM); out.push(*v); }
            Instr::Load(a)      => { out.push(LOAD);     out.push(*a); }
            Instr::Store(a)     => { out.push(STORE);    out.push(*a); }
            Instr::Store15(a)   => { out.push(STORE15);  out.push(*a); }
            Instr::Jump(off)    => { out.push(JUMP);     out.push(*off as u16); }
            Instr::JumpIf(off)  => { out.push(JUMP_IF);  out.push(*off as u16); }
            Instr::JumpNot(off) => { out.push(JUMP_NOT); out.push(*off as u16); }
            Instr::SetOff(v)    => { out.push(SET_OFF);  out.push(*v); }
            Instr::Lshr(k)      => { out.push(LSHR);     out.push(*k); }
            Instr::Lshl(k)      => { out.push(LSHL);     out.push(*k); }
            Instr::HostCall { slot, n_args, n_results } => {
                out.push(HOST_CALL);
                out.push(*slot);
                out.push(((*n_args as u16) << 8) | (*n_results as u16));
            }
        }
    }

    /// Decode one instruction from `words[offset..]`.
    ///
    /// Returns `Some((instr, words_consumed))`, or `None` if the offset is out of
    /// bounds or the opcode is unknown.
    pub fn decode(words: &[u16], offset: usize) -> Option<(Self, usize)> {
        use op::*;
        let opc = *words.get(offset)?;
        let op1 = |i: usize| -> Option<u16> { words.get(i).copied() };

        let instr = match opc {
            RET            => Instr::Ret,
            ADD            => Instr::Add,
            SUB            => Instr::Sub,
            AND            => Instr::And,
            OR             => Instr::Or,
            XOR            => Instr::Xor,
            NOT            => Instr::Not,
            MASK15         => Instr::Mask15,
            NEG            => Instr::Neg,
            LSHR_STK       => Instr::LshrStk,
            LSHL_STK       => Instr::LshlStk,
            IMUL_HI15      => Instr::ImulHi15,
            IMUL_LO15      => Instr::ImulLo15,
            IDIV_Q15       => Instr::IdivQ15,
            IDIV_R15       => Instr::IdivR15,
            IS_POS         => Instr::IsPos,
            IS_PLUS_ZERO   => Instr::IsPlusZero,
            IS_NEG         => Instr::IsNeg,
            IS_MINUS_ZERO  => Instr::IsMinusZero,
            IS_ZERO_OR_NEG => Instr::IsZeroOrNeg,
            HAS_OVERFLOW   => Instr::HasOverflow,
            BOOL_AND       => Instr::BoolAnd,
            BOOL_NOT       => Instr::BoolNot,
            DUP            => Instr::Dup,
            SWAP           => Instr::Swap,
            DROP           => Instr::Drop,
            LOAD_T         => Instr::LoadT,
            STORE_T        => Instr::StoreT,
            LOAD_OFF       => Instr::LoadOff,
            STORE_OFF      => Instr::StoreOff,
            LOAD_OFF1      => Instr::LoadOff1,
            STORE_OFF1     => Instr::StoreOff1,
            GET_OFF        => Instr::GetOff,
            SET_OFF_STACK  => Instr::SetOffStack,
            LOAD_CHAN_OFF   => Instr::LoadChanOff,
            STORE_CHAN_OFF  => Instr::StoreChanOff,
            LOAD_IND       => Instr::LoadInd,
            STORE_IND      => Instr::StoreInd,
            PUSH_IMM       => return Some((Instr::PushImm(op1(offset + 1)?), 2)),
            LOAD           => return Some((Instr::Load(op1(offset + 1)?), 2)),
            STORE          => return Some((Instr::Store(op1(offset + 1)?), 2)),
            STORE15        => return Some((Instr::Store15(op1(offset + 1)?), 2)),
            JUMP           => return Some((Instr::Jump(op1(offset + 1)? as i16), 2)),
            JUMP_IF        => return Some((Instr::JumpIf(op1(offset + 1)? as i16), 2)),
            JUMP_NOT       => return Some((Instr::JumpNot(op1(offset + 1)? as i16), 2)),
            SET_OFF        => return Some((Instr::SetOff(op1(offset + 1)?), 2)),
            LSHR           => return Some((Instr::Lshr(op1(offset + 1)?), 2)),
            LSHL           => return Some((Instr::Lshl(op1(offset + 1)?), 2)),
            HOST_CALL      => {
                let slot = op1(offset + 1)?;
                let packed = op1(offset + 2)?;
                let n_args    = (packed >> 8) as u8;
                let n_results = (packed & 0xFF) as u8;
                return Some((Instr::HostCall { slot, n_args, n_results }, 3));
            }
            _              => return None,
        };
        Some((instr, 1))
    }
}
