//! Instruction execution — evaluates [`SemOp`] sequences against [`Cpu`] state.
//!
//! The interpreter is driven entirely by the `semantics` field of each
//! [`InstrSpec`].  For each decoded instruction:
//!
//! 1. Retrieve the `Vec<SemOp>` from the spec.
//! 2. Evaluate each `SemOp` in order via [`eval_op`].
//! 3. Unless a `Branch` / `BranchCond` / special op redirected Z, advance Z by 1.
//!
//! This makes the interpreter fully data-driven: adding a new instruction only
//! requires writing its semantics in the text DSL and inserting it into the spec set.

use crate::cpu::{
    self, Cpu, BRUPT_ADDR, ZRUPT_ADDR,
};
use crate::decode::DecodedInstr;
use crate::semantics::{Cond, Dest, Expr, Flag, SemOp};
use crate::spec::InstrType;

// ─── Execution error ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ExecError {
    /// No semantics defined for this instruction type.
    NoSemantics(InstrType),
    /// Division by zero.
    DivideByZero,
    /// Attempted to execute a halting/undefined instruction.
    Halt(InstrType),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::NoSemantics(t)  => write!(f, "no semantics for {t}"),
            ExecError::DivideByZero    => f.write_str("division by zero"),
            ExecError::Halt(t)         => write!(f, "halt: {t}"),
        }
    }
}

impl std::error::Error for ExecError {}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Execute a single decoded instruction on `cpu`.
///
/// The `ops` slice should come from `instr.spec.semantics` (already parsed).
/// Pass `None` to fall back to special-case handling for CCS / RESUME / RUPT.
///
/// After execution, Z points to the *next* instruction to fetch.
pub fn execute(cpu: &mut Cpu, instr: &DecodedInstr<'_>, ops: &[SemOp]) -> Result<(), ExecError> {
    // Save pre-execution Z (= address of this instruction, already fetched).
    // Z has already been incremented by fetch in the main loop, so Z now points
    // to the *next* word.  We pass the operand address separately.
    let operand = instr.address;

    let mut pc_overridden = false;

    // Handle atomic-swap instructions before the op loop (XCH, LXCH, QXCH, DXCH).
    // These need to capture the old memory value before writing.
    let ty = instr.spec.instr_type;
    if matches!(ty, InstrType::Xch | InstrType::Lxch | InstrType::Qxch | InstrType::Dxch) {
        execute_exchange(cpu, ty, operand);
        // PC already advanced by caller; nothing more to do.
        return Ok(());
    }

    // Handle MP which reads A before writing both A and L.
    if ty == InstrType::Mp {
        let a = cpu.a;
        let m = cpu.mem_read(operand);
        let (hi, lo) = cpu::oc_mul(a, m);
        cpu.a = hi;
        cpu.l = lo;
        return Ok(());
    }

    // Handle DV (divide double-precision A,L by E).
    if ty == InstrType::Dv {
        let a_hi = cpu.a;
        let a_lo = cpu.l;
        let div  = cpu.mem_read(operand);
        match cpu::oc_div(a_hi, a_lo, div) {
            Some((q, r)) => { cpu.a = q; cpu.l = r; }
            None          => return Err(ExecError::DivideByZero),
        }
        return Ok(());
    }

    // Handle DAS (double-precision add-to-storage) — uses both mem and mem+1.
    if ty == InstrType::Das {
        execute_das(cpu, operand);
        return Ok(());
    }

    // Handle DCA / DCS (double-precision load/negate-load).
    if ty == InstrType::Dca {
        cpu.l = cpu.mem_read(operand);
        cpu.a = cpu.mem_read(operand.wrapping_add(1));
        return Ok(());
    }
    if ty == InstrType::Dcs {
        cpu.l = cpu::oc_sub(0, cpu.mem_read(operand)).0;
        cpu.a = cpu::oc_sub(0, cpu.mem_read(operand.wrapping_add(1))).0;
        return Ok(());
    }

    // Handle NDX (index) — adds operand word to the next instruction's address field.
    if ty == InstrType::Ndx {
        let index = cpu.mem_read(operand);
        apply_index(cpu, index);
        return Ok(());
    }

    // Evaluate generic SemOp sequence.
    for op in ops {
        match op {
            SemOp::Set(dest, expr) => {
                let val = eval_expr(cpu, operand, expr)?;
                write_dest(cpu, *dest, val);
            }

            SemOp::Branch(expr) => {
                let target = eval_expr(cpu, operand, expr)? & 0x7FFF;
                cpu.z = target;
                pc_overridden = true;
            }

            SemOp::BranchCond { cond, taken } => {
                if eval_cond(cpu, cond) {
                    let target = eval_expr(cpu, operand, taken)? & 0x7FFF;
                    cpu.z = target;
                } // else: Z already points to next instruction (incremented by caller)
                pc_overridden = true; // either way, no further increment
            }

            SemOp::CcsBranch => {
                execute_ccs(cpu, operand);
                pc_overridden = true;
            }

            SemOp::SetFlag(fl) => set_flag(cpu, *fl),
            SemOp::ClearFlag(fl) => clear_flag(cpu, *fl),

            SemOp::Interrupt => {
                execute_rupt(cpu);
                pc_overridden = true;
            }

            SemOp::Resume => {
                execute_resume(cpu);
                pc_overridden = true;
            }
        }
    }

    // If EXTEND was just set, do NOT clear it here — it remains for the next instruction.
    // The main loop clears it after the following instruction executes.
    // (EXTEND is self-clearing in the main loop, not here.)

    let _ = pc_overridden; // caller (interp) already advanced Z before calling us
    Ok(())
}

// ─── Expression evaluator ─────────────────────────────────────────────────────

fn eval_expr(cpu: &Cpu, operand: u16, expr: &Expr) -> Result<u16, ExecError> {
    let r = match expr {
        Expr::Lit(n)        => *n,
        Expr::A             => cpu.a,
        Expr::L             => cpu.l,
        Expr::Q             => cpu.q,
        Expr::Z             => cpu.z,
        Expr::Mem           => cpu.mem_read(operand),
        Expr::MemHi         => cpu.mem_read(operand.wrapping_add(1)),
        Expr::Channel       => cpu.channel_read(operand),
        Expr::Operand       => operand,
        Expr::OcAdd(a, b)   => cpu::oc_add(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).0,
        Expr::OcSub(a, b)   => cpu::oc_sub(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).0,
        Expr::OcNeg(x)      => (!eval_expr(cpu, operand, x)?) & 0x7FFF,
        Expr::And(a, b)     => eval_expr(cpu, operand, a)? & eval_expr(cpu, operand, b)?,
        Expr::Or(a, b)      => eval_expr(cpu, operand, a)? | eval_expr(cpu, operand, b)?,
        Expr::Xor(a, b)     => eval_expr(cpu, operand, a)? ^ eval_expr(cpu, operand, b)?,
        Expr::Not(x)        => (!eval_expr(cpu, operand, x)?) & 0x7FFF,
        Expr::MulHi(a, b)   => cpu::oc_mul(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).0,
        Expr::MulLo(a, b)   => cpu::oc_mul(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).1,
        Expr::DivQ(a, b)    => {
            let b_val = eval_expr(cpu, operand, b)?;
            if b_val == 0 || b_val == 0x7FFF { return Err(ExecError::DivideByZero); }
            cpu::oc_div(eval_expr(cpu, operand, a)?, cpu.l, b_val)
                .ok_or(ExecError::DivideByZero)?.0
        }
        Expr::DivR(a, b)    => {
            let b_val = eval_expr(cpu, operand, b)?;
            if b_val == 0 || b_val == 0x7FFF { return Err(ExecError::DivideByZero); }
            cpu::oc_div(eval_expr(cpu, operand, a)?, cpu.l, b_val)
                .ok_or(ExecError::DivideByZero)?.1
        }
        Expr::Augment(x)    => cpu::oc_augment(eval_expr(cpu, operand, x)?),
        Expr::Diminish(x)   => cpu::oc_diminish(eval_expr(cpu, operand, x)?),
    };
    Ok(r)
}

// ─── Destination writer ───────────────────────────────────────────────────────

fn write_dest(cpu: &mut Cpu, dest: Dest, val: u16) {
    match dest {
        Dest::A       => { cpu.a = val; }
        Dest::L       => { cpu.l = val & 0x7FFF; }
        Dest::Q       => { cpu.q = val & 0x7FFF; }
        Dest::Z       => { cpu.z = val & 0x7FFF; }
        Dest::Mem     => { /* needs operand — handled by caller */ }
        Dest::MemHi   => { /* needs operand — handled by caller */ }
        Dest::Channel => { /* needs operand — handled by caller */ }
    }
}

// The op loop uses these specialised writers when operand is in scope.
fn write_dest_with_addr(cpu: &mut Cpu, dest: Dest, val: u16, operand: u16) {
    match dest {
        Dest::Mem     => cpu.mem_write(operand, val & 0x7FFF),
        Dest::MemHi   => cpu.mem_write(operand.wrapping_add(1), val & 0x7FFF),
        Dest::Channel => cpu.channel_write(operand, val & 0x7FFF),
        other         => write_dest(cpu, other, val),
    }
}

// ─── Condition evaluator ──────────────────────────────────────────────────────

fn eval_cond(cpu: &Cpu, cond: &Cond) -> bool {
    let a15 = cpu.a & 0x7FFF;
    match cond {
        Cond::APos        => cpu::is_positive(a15),
        Cond::APlusZero   => cpu::is_plus_zero(a15),
        Cond::ANeg        => cpu::is_negative(a15),
        Cond::AMinusZero  => cpu::is_minus_zero(a15),
        Cond::AZeroOrNeg  => cpu::is_plus_zero(a15) || cpu::is_minus_zero(a15) || cpu::is_negative(a15),
        Cond::AOverflow   => cpu::accumulator_overflow(cpu.a) != (false, false),
        Cond::And(a, b)   => eval_cond(cpu, a) && eval_cond(cpu, b),
        Cond::Not(c)      => !eval_cond(cpu, c),
    }
}

// ─── Control flag helpers ─────────────────────────────────────────────────────

fn set_flag(cpu: &mut Cpu, fl: Flag) {
    match fl {
        Flag::Extend => cpu.extend = true,
        Flag::Inhint => cpu.inhint = true,
    }
}

fn clear_flag(cpu: &mut Cpu, fl: Flag) {
    match fl {
        Flag::Extend => cpu.extend = false,
        Flag::Inhint => cpu.inhint = false,
    }
}

// ─── Specialised instruction helpers ─────────────────────────────────────────

/// CCS — Count, Compare, and Skip.
///
/// Reads E, stores it in A, then branches based on the 4-way comparison.
fn execute_ccs(cpu: &mut Cpu, e_addr: u16) {
    let val = cpu.mem_read(e_addr);
    cpu.a = val;
    // Z currently points to instruction following CCS (already incremented by fetch).
    // Skip 0, 1, 2, or 3 additional instructions for positive / +zero / negative / -zero.
    let skip: u16 = if cpu::is_positive(val) {
        0
    } else if cpu::is_plus_zero(val) {
        1
    } else if cpu::is_negative(val) {
        2
    } else {
        // Minus-zero
        3
    };
    cpu.z = cpu.z.wrapping_add(skip) & 0x7FFF;
}

/// TS — Transfer to Storage with overflow branch.
///
/// Stores A in E.  If A has overflow, skips the next instruction.
pub fn execute_ts(cpu: &mut Cpu, e_addr: u16) {
    cpu.mem_write(e_addr, cpu.a & 0x7FFF);
    let (pos_ov, neg_ov) = cpu::accumulator_overflow(cpu.a);
    if pos_ov || neg_ov {
        // Skip one instruction (Z already incremented; add 1 more)
        cpu.z = cpu.z.wrapping_add(1) & 0x7FFF;
        // Saturate A to ±max
        cpu.a = if pos_ov { 0x3FFF } else { 0x4000 };
    }
}

/// Atomic exchange: reg ↔ memory[addr].
fn execute_exchange(cpu: &mut Cpu, ty: InstrType, addr: u16) {
    let mem_val = cpu.mem_read(addr);
    match ty {
        InstrType::Xch  => { cpu.mem_write(addr, cpu.a & 0x7FFF); cpu.a = mem_val; }
        InstrType::Lxch => { cpu.mem_write(addr, cpu.l); cpu.l = mem_val; }
        InstrType::Qxch => { cpu.mem_write(addr, cpu.q); cpu.q = mem_val; }
        InstrType::Dxch => {
            // Atomic double exchange: (A,L) ↔ (mem[addr], mem[addr+1])
            let mem_hi = cpu.mem_read(addr.wrapping_add(1));
            cpu.mem_write(addr, cpu.a & 0x7FFF);
            cpu.mem_write(addr.wrapping_add(1), cpu.l);
            cpu.a = mem_val;
            cpu.l = mem_hi;
        }
        _ => {}
    }
}

/// DAS — Double-precision Add to Storage.
fn execute_das(cpu: &mut Cpu, e_addr: u16) {
    let a = cpu.a;
    let l = cpu.l;
    let mem_lo = cpu.mem_read(e_addr);
    let mem_hi = cpu.mem_read(e_addr.wrapping_add(1));

    let (lo_sum, lo_carry) = cpu::oc_add(l, mem_lo);
    let carry_word: u16 = if lo_carry { 1 } else { 0 };
    let (hi_sum, _) = cpu::oc_add(cpu::oc_add(a, mem_hi).0, carry_word);

    cpu.mem_write(e_addr, lo_sum);
    cpu.mem_write(e_addr.wrapping_add(1), hi_sum);
    // A gets overflow indicator (0 if no overflow)
    cpu.a = 0;
}

/// NDX — Apply index to the *next* instruction's address field.
///
/// Adds `index` to bits [11:0] of the next instruction word (preserving the opcode).
fn apply_index(cpu: &mut Cpu, index: u16) {
    let next_addr = cpu.z; // Z already points to next instruction after fetch
    let next_word = cpu.mem_read(next_addr);
    // Add index to address field (lower 12 bits), let carry into opcode (AGC behaviour)
    let indexed = next_word.wrapping_add(index & 0x7FFF) & 0x7FFF;
    // Write the modified word back so the next fetch picks it up.
    // (In hardware this happens in the instruction register, not memory;
    //  here we patch the memory word for simplicity.)
    cpu.mem_write(next_addr, indexed);
}

/// RUPT — Save state and vector to interrupt handler.
fn execute_rupt(cpu: &mut Cpu) {
    // Save current instruction (BRUPT) and PC (ZRUPT)
    cpu.mem_write(BRUPT_ADDR, cpu.mem_read(cpu.z));
    cpu.mem_write(ZRUPT_ADDR, cpu.z);
    // Vector: the interrupt address would normally come from priority logic;
    // use a placeholder that the platform can override via channel conventions.
    // For now, branch to a configurable interrupt vector stored in the interrupt
    // vector table at low erasable memory (address 0o4).
    let vector = cpu.mem_read(0o4);
    cpu.z = vector & 0x7FFF;
}

/// RESUME — Restore state from interrupt save registers.
fn execute_resume(cpu: &mut Cpu) {
    cpu.z = cpu.mem_read(ZRUPT_ADDR);
    // The BRUPT word contains the instruction to re-execute; it has already
    // been saved to memory.  The next fetch will read it from Z.
}

// ─── Generic SemOp evaluator (used by interp for Set ops with operand) ────────

/// Full-operand-aware op evaluator.  Used by the main loop instead of `execute`
/// to handle `Dest::Mem` / `Dest::Channel` correctly.
pub fn eval_op(cpu: &mut Cpu, operand: u16, op: &SemOp) -> Result<Option<u16>, ExecError> {
    // Returns Some(new_Z) if Z was redirected, None otherwise.
    match op {
        SemOp::Set(dest, expr) => {
            let val = eval_expr(cpu, operand, expr)?;
            write_dest_with_addr(cpu, *dest, val, operand);
            Ok(None)
        }
        SemOp::Branch(expr) => {
            let target = eval_expr(cpu, operand, expr)?;
            Ok(Some(target & 0x7FFF))
        }
        SemOp::BranchCond { cond, taken } => {
            if eval_cond(cpu, cond) {
                let target = eval_expr(cpu, operand, taken)?;
                Ok(Some(target & 0x7FFF))
            } else {
                Ok(None)
            }
        }
        SemOp::CcsBranch => {
            execute_ccs(cpu, operand);
            Ok(Some(cpu.z)) // CCS has already updated Z
        }
        SemOp::SetFlag(fl)   => { set_flag(cpu, *fl);   Ok(None) }
        SemOp::ClearFlag(fl) => { clear_flag(cpu, *fl); Ok(None) }
        SemOp::Interrupt     => { execute_rupt(cpu);   Ok(Some(cpu.z)) }
        SemOp::Resume        => { execute_resume(cpu); Ok(Some(cpu.z)) }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Cpu;
    use crate::semantics::parse_sem;

    #[test]
    fn exec_ca() {
        let mut cpu = Cpu::new();
        cpu.mem_write(0o100, 42);
        let ops = parse_sem("set A mem").unwrap();
        let operand = 0o100u16;
        for op in &ops { eval_op(&mut cpu, operand, op).unwrap(); }
        assert_eq!(cpu.a, 42);
    }

    #[test]
    fn exec_ad() {
        let mut cpu = Cpu::new();
        cpu.a = 10;
        cpu.mem_write(0o100, 5);
        let ops = parse_sem("set A oc_add(A,mem)").unwrap();
        for op in &ops { eval_op(&mut cpu, 0o100, op).unwrap(); }
        assert_eq!(cpu.a, 15);
    }

    #[test]
    fn exec_tc_saves_q() {
        let mut cpu = Cpu::new();
        cpu.z = 0o200;
        let ops = parse_sem("set Q Z\nbranch operand").unwrap();
        let mut new_z = None;
        for op in &ops {
            if let Some(z) = eval_op(&mut cpu, 0o300, op).unwrap() {
                new_z = Some(z);
            }
        }
        assert_eq!(cpu.q, 0o200);  // return address saved
        assert_eq!(new_z, Some(0o300));
    }

    #[test]
    fn exec_bzf_taken() {
        let mut cpu = Cpu::new();
        cpu.a = 0; // plus-zero
        cpu.z = 0o100;
        let ops = parse_sem("branch_if A_plus_zero operand").unwrap();
        let mut new_z = None;
        for op in &ops {
            if let Some(z) = eval_op(&mut cpu, 0o500, op).unwrap() {
                new_z = Some(z);
            }
        }
        assert_eq!(new_z, Some(0o500)); // branch taken
    }

    #[test]
    fn exec_bzf_not_taken() {
        let mut cpu = Cpu::new();
        cpu.a = 1; // positive, not zero
        let ops = parse_sem("branch_if A_plus_zero operand").unwrap();
        let result = eval_op(&mut cpu, 0o500, &ops[0]).unwrap();
        assert_eq!(result, None); // branch not taken
    }

    #[test]
    fn exec_extend_flag() {
        let mut cpu = Cpu::new();
        let ops = parse_sem("set_flag extend").unwrap();
        eval_op(&mut cpu, 0, &ops[0]).unwrap();
        assert!(cpu.extend);
    }

    #[test]
    fn exec_xch() {
        let mut cpu = Cpu::new();
        cpu.a = 0o777;
        cpu.mem_write(0o100, 0o123);
        execute_exchange(&mut cpu, InstrType::Xch, 0o100);
        assert_eq!(cpu.a, 0o123);
        assert_eq!(cpu.mem_read(0o100), 0o777);
    }
}
