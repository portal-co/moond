//! Instruction execution — evaluates [`SemOp`] sequences against [`Cpu`] state.
//!
//! The interpreter is fully data-driven: each instruction's behavior is expressed
//! as a `Vec<SemOp>` and evaluated in order. Execution short-circuits after the
//! first branch taken.

use agc_isa::{Cond, Dest, Expr, Flag, SemOp};
use agc_isa::InstrType;

use crate::cpu::{self, Cpu};

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

impl core::fmt::Display for ExecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExecError::NoSemantics(t) => write!(f, "no semantics for {t}"),
            ExecError::DivideByZero   => f.write_str("division by zero"),
            ExecError::Halt(t)        => write!(f, "halt: {t}"),
        }
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Execute a sequence of SemOps against CPU state.
///
/// Returns `Ok(true)` if a branch was taken (Z was updated), `Ok(false)` otherwise.
/// Short-circuits after the first branch taken.
pub fn exec_ops(cpu: &mut Cpu, operand: u16, ops: &[SemOp]) -> Result<bool, ExecError> {
    for op in ops {
        match op {
            SemOp::Set(dest, expr) => {
                let val = eval_expr(cpu, operand, expr)?;
                write_dest(cpu, dest, val, operand);
            }

            SemOp::Branch(expr) => {
                let target = eval_expr(cpu, operand, expr)? & 0x7FFF;
                cpu.z = target;
                return Ok(true); // short-circuit
            }

            SemOp::BranchIf(cond, expr) => {
                if eval_cond(cpu, operand, cond) {
                    let target = eval_expr(cpu, operand, expr)? & 0x7FFF;
                    cpu.z = target;
                    return Ok(true); // short-circuit
                }
                // Not taken — Z already points to next instruction (advanced by caller)
            }

            SemOp::SetFlag(fl)   => set_flag(cpu, *fl),
            SemOp::ClearFlag(fl) => clear_flag(cpu, *fl),
        }
    }
    Ok(false)
}

// ─── Expression evaluator ─────────────────────────────────────────────────────

/// Evaluate an expression against LIVE CPU state (reads happen at evaluation time).
pub fn eval_expr(cpu: &Cpu, operand: u16, expr: &Expr) -> Result<u16, ExecError> {
    let r = match expr {
        Expr::Lit(n)              => *n,
        Expr::A                   => cpu.a,
        Expr::L                   => cpu.l,
        Expr::Q                   => cpu.q,
        Expr::Z                   => cpu.z,
        Expr::Tmp                 => cpu.tmp,
        Expr::Mem                 => cpu.mem_read(operand),
        Expr::MemHi               => cpu.mem_read(operand.wrapping_add(1)),
        Expr::Channel             => cpu.channel_read(operand),
        Expr::Operand             => operand,
        Expr::InstrWord           => cpu.instr_word,
        Expr::PcRel(offset) => {
            ((cpu.z as i32) + (*offset as i32)).max(0) as u16 & 0x7FFF
        }
        Expr::MemAt(addr) => cpu.mem_read(*addr),
        Expr::Deref(e) => {
            let addr = eval_expr(cpu, operand, e)?;
            cpu.mem_read(addr)
        }

        // Ones-complement arithmetic
        Expr::OcAdd(a, b)   => cpu::oc_add(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).0,
        Expr::OcSub(a, b)   => cpu::oc_sub(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).0,
        Expr::OcNeg(x)      => (!eval_expr(cpu, operand, x)?) & 0x7FFF,

        // Bitwise
        Expr::And(a, b) => eval_expr(cpu, operand, a)? & eval_expr(cpu, operand, b)?,
        Expr::Or(a, b)  => eval_expr(cpu, operand, a)? | eval_expr(cpu, operand, b)?,
        Expr::Xor(a, b) => eval_expr(cpu, operand, a)? ^ eval_expr(cpu, operand, b)?,
        Expr::Not(x)    => (!eval_expr(cpu, operand, x)?) & 0x7FFF,

        // Double-precision
        Expr::MulHi(a, b) => cpu::oc_mul(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).0,
        Expr::MulLo(a, b) => cpu::oc_mul(eval_expr(cpu, operand, a)?, eval_expr(cpu, operand, b)?).1,

        Expr::DivQ(a, b, c) => {
            let a_val = eval_expr(cpu, operand, a)?;
            let b_val = eval_expr(cpu, operand, b)?;
            let c_val = eval_expr(cpu, operand, c)?;
            cpu::oc_div(a_val, b_val, c_val)
                .ok_or(ExecError::DivideByZero)?.0
        }
        Expr::DivR(a, b, c) => {
            let a_val = eval_expr(cpu, operand, a)?;
            let b_val = eval_expr(cpu, operand, b)?;
            let c_val = eval_expr(cpu, operand, c)?;
            cpu::oc_div(a_val, b_val, c_val)
                .ok_or(ExecError::DivideByZero)?.1
        }

        Expr::DpAddHi(a, b, c, d) => {
            let a_val = eval_expr(cpu, operand, a)?;
            let b_val = eval_expr(cpu, operand, b)?;
            let c_val = eval_expr(cpu, operand, c)?;
            let d_val = eval_expr(cpu, operand, d)?;
            cpu::dp_add_hi(a_val, b_val, c_val, d_val)
        }

        // Augment/Diminish/Saturate
        Expr::Augment(x)  => cpu::oc_augment(eval_expr(cpu, operand, x)?),
        Expr::Diminish(x) => cpu::oc_diminish(eval_expr(cpu, operand, x)?),
        Expr::Saturate(x) => cpu::oc_saturate(eval_expr(cpu, operand, x)?),
    };
    Ok(r)
}

// ─── Destination writer ───────────────────────────────────────────────────────

/// Write a value to a destination, with operand address in scope.
pub fn write_dest(cpu: &mut Cpu, dest: &Dest, val: u16, operand: u16) {
    match dest {
        Dest::A       => { cpu.a = val; }
        Dest::L       => { cpu.l = val & 0x7FFF; }
        Dest::Q       => { cpu.q = val & 0x7FFF; }
        Dest::Z       => { cpu.z = val & 0x7FFF; }
        Dest::Tmp     => { cpu.tmp = val; }
        Dest::Mem     => cpu.mem_write(operand, val & 0x7FFF),
        Dest::MemHi   => cpu.mem_write(operand.wrapping_add(1), val & 0x7FFF),
        Dest::Channel => cpu.channel_write(operand, val & 0x7FFF),
        Dest::MemAt(addr) => cpu.mem_write(*addr, val & 0x7FFF),
        Dest::Deref(e) => {
            // Compute address from expression — use a snapshot of tmp/z
            // We need a read-only cpu to compute the address; use a helper
            let addr = eval_dest_addr(cpu, operand, e);
            cpu.mem_write(addr, val & 0x7FFF);
        }
    }
}

/// Evaluate a Deref expression to get the address (read-only helper).
fn eval_dest_addr(cpu: &Cpu, operand: u16, expr: &Expr) -> u16 {
    eval_expr(cpu, operand, expr).unwrap_or(0) & 0x7FFF
}

// ─── Condition evaluator ──────────────────────────────────────────────────────

/// Evaluate a condition against LIVE CPU state.
pub fn eval_cond(cpu: &Cpu, operand: u16, cond: &Cond) -> bool {
    match cond {
        Cond::IsPositive(e)  => {
            let v = eval_expr(cpu, operand, e).unwrap_or(0) & 0x7FFF;
            cpu::is_positive(v)
        }
        Cond::IsPlusZero(e) => {
            let v = eval_expr(cpu, operand, e).unwrap_or(0) & 0x7FFF;
            cpu::is_plus_zero(v)
        }
        Cond::IsNegative(e) => {
            let v = eval_expr(cpu, operand, e).unwrap_or(0) & 0x7FFF;
            cpu::is_negative(v)
        }
        Cond::IsMinusZero(e) => {
            let v = eval_expr(cpu, operand, e).unwrap_or(0) & 0x7FFF;
            cpu::is_minus_zero(v)
        }
        Cond::IsZeroOrNeg(e) => {
            let v = eval_expr(cpu, operand, e).unwrap_or(0) & 0x7FFF;
            cpu::is_plus_zero(v) || cpu::is_minus_zero(v) || cpu::is_negative(v)
        }
        Cond::HasOverflow(e) => {
            // HasOverflow takes the full 16-bit value from A
            let v = eval_expr(cpu, operand, e).unwrap_or(0);
            cpu::has_overflow(v)
        }
        Cond::And(a, b) => eval_cond(cpu, operand, a) && eval_cond(cpu, operand, b),
        Cond::Not(c)    => !eval_cond(cpu, operand, c),
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agc_isa::parse_sem;

    #[test]
    fn exec_ca() {
        let mut cpu = Cpu::new();
        cpu.mem_write(0o100, 42);
        let ops = parse_sem("set A mem").unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        assert_eq!(cpu.a, 42);
    }

    #[test]
    fn exec_ad() {
        let mut cpu = Cpu::new();
        cpu.a = 10;
        cpu.mem_write(0o100, 5);
        let ops = parse_sem("set A oc_add(A,mem)").unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        assert_eq!(cpu.a, 15);
    }

    #[test]
    fn exec_tc_saves_q() {
        let mut cpu = Cpu::new();
        cpu.z = 0o200;
        let ops = parse_sem("set Q Z\nbranch operand").unwrap();
        let branched = exec_ops(&mut cpu, 0o300, &ops).unwrap();
        assert_eq!(cpu.q, 0o200);
        assert!(branched);
        assert_eq!(cpu.z, 0o300);
    }

    #[test]
    fn exec_bzf_taken() {
        let mut cpu = Cpu::new();
        cpu.a = 0; // plus-zero
        let ops = parse_sem("branch_if is_plus_zero(A) operand").unwrap();
        let branched = exec_ops(&mut cpu, 0o500, &ops).unwrap();
        assert!(branched);
        assert_eq!(cpu.z, 0o500);
    }

    #[test]
    fn exec_bzf_not_taken() {
        let mut cpu = Cpu::new();
        cpu.a = 1; // positive, not zero
        cpu.z = 0o100;
        let ops = parse_sem("branch_if is_plus_zero(A) operand").unwrap();
        let branched = exec_ops(&mut cpu, 0o500, &ops).unwrap();
        assert!(!branched);
        assert_eq!(cpu.z, 0o100); // unchanged
    }

    #[test]
    fn exec_extend_flag() {
        let mut cpu = Cpu::new();
        let ops = parse_sem("set_flag extend").unwrap();
        exec_ops(&mut cpu, 0, &ops).unwrap();
        assert!(cpu.extend);
    }

    #[test]
    fn exec_xch() {
        let mut cpu = Cpu::new();
        cpu.a = 0o777;
        cpu.mem_write(0o100, 0o123);
        let ops = parse_sem("set tmp mem\nset mem A\nset A tmp").unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        assert_eq!(cpu.a, 0o123);
        assert_eq!(cpu.mem_read(0o100), 0o777);
    }

    #[test]
    fn exec_ccs_positive() {
        let mut cpu = Cpu::new();
        cpu.mem_write(0o100, 5); // positive
        cpu.z = 0o200; // Z already advanced past CCS (so Z = addr of first skip slot)
        let ops = parse_sem(
            "set A mem\n\
             branch_if is_pos(A) pc_rel(0)\n\
             branch_if is_plus_zero(A) pc_rel(1)\n\
             branch_if is_neg(A) pc_rel(2)\n\
             branch pc_rel(3)"
        ).unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        assert_eq!(cpu.a, 5);
        assert_eq!(cpu.z, 0o200); // pc_rel(0) = Z + 0 = 0o200
    }

    #[test]
    fn exec_ccs_plus_zero() {
        let mut cpu = Cpu::new();
        cpu.mem_write(0o100, 0); // plus-zero
        cpu.z = 0o200;
        let ops = parse_sem(
            "set A mem\n\
             branch_if is_pos(A) pc_rel(0)\n\
             branch_if is_plus_zero(A) pc_rel(1)\n\
             branch_if is_neg(A) pc_rel(2)\n\
             branch pc_rel(3)"
        ).unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        assert_eq!(cpu.z, 0o201); // pc_rel(1) = Z + 1 = 0o201
    }

    #[test]
    fn exec_ts_overflow() {
        let mut cpu = Cpu::new();
        // Positive overflow: A has bit15=0, bit14=1
        cpu.a = 0b0100_0000_0000_0001u16; // positive overflow
        cpu.z = 0o200;
        let ops = parse_sem(
            "set tmp A\n\
             set mem A\n\
             set A sat(tmp)\n\
             branch_if has_overflow(tmp) pc_rel(1)"
        ).unwrap();
        exec_ops(&mut cpu, 0o50, &ops).unwrap();
        assert_eq!(cpu.a, 0x3FFF); // saturated to +max
        assert_eq!(cpu.z, 0o201); // skipped one instruction
    }

    #[test]
    fn exec_dv() {
        let mut cpu = Cpu::new();
        cpu.a = 0o0;    // a_hi
        cpu.l = 10;     // a_lo
        cpu.mem_write(0o100, 3); // divisor
        // DV: set tmp A; set A div_q(tmp,L,mem); set L div_r(tmp,L,mem)
        let ops = parse_sem(
            "set tmp A\nset A div_q(tmp,L,mem)\nset L div_r(tmp,L,mem)"
        ).unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        assert_eq!(cpu.a, 3); // quotient
        assert_eq!(cpu.l, 1); // remainder
    }

    #[test]
    fn exec_mp() {
        let mut cpu = Cpu::new();
        cpu.a = 3;
        cpu.mem_write(0o100, 4);
        let ops = parse_sem("set L mul_lo(A,mem)\nset A mul_hi(A,mem)").unwrap();
        exec_ops(&mut cpu, 0o100, &ops).unwrap();
        // 3 * 4 = 12; fits in lo word
        assert_eq!(cpu.l, 12);
        assert_eq!(cpu.a, 0);
    }

    #[test]
    fn exec_short_circuit_after_branch() {
        // Verify that after a Branch, subsequent Set ops are NOT executed
        let mut cpu = Cpu::new();
        cpu.z = 0o100;
        cpu.a = 0;
        let ops = parse_sem("branch operand\nset A 0o777").unwrap();
        exec_ops(&mut cpu, 0o500, &ops).unwrap();
        assert_eq!(cpu.z, 0o500); // branch taken
        assert_eq!(cpu.a, 0);     // set A was NOT executed
    }
}
