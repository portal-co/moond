//! High-level interpreter: fetch → decode → execute loop.

extern crate alloc;

use agc_isa::{InstrSpecSet, InstrType, builtin_spec_set};
use crate::cpu::Cpu;
use crate::decode;
use crate::exec::{self, ExecError};

// ─── Interpreter error ────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum InterpError {
    Decode(decode::DecodeError),
    Exec(ExecError),
    /// Execution halted.
    Halt,
}

impl core::fmt::Display for InterpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InterpError::Decode(e) => write!(f, "decode error: {e}"),
            InterpError::Exec(e)   => write!(f, "exec error: {e}"),
            InterpError::Halt      => f.write_str("halt"),
        }
    }
}

impl From<decode::DecodeError> for InterpError {
    fn from(e: decode::DecodeError) -> Self { InterpError::Decode(e) }
}

impl From<ExecError> for InterpError {
    fn from(e: ExecError) -> Self { InterpError::Exec(e) }
}

// ─── Interpreter ─────────────────────────────────────────────────────────────

/// The AGC Block-2 interpreter.
pub struct Interpreter {
    cpu: Cpu,
    specs: InstrSpecSet,
    /// Instruction cycle counter.
    pub cycle_count: u64,
}

impl Interpreter {
    /// Create a new interpreter from an `InstrSpecSet`.
    pub fn new(specs: InstrSpecSet) -> Self {
        Interpreter {
            cpu: Cpu::new(),
            specs,
            cycle_count: 0,
        }
    }

    /// Create an interpreter using the built-in ISA table.
    pub fn with_builtin_specs() -> Self {
        Self::new(builtin_spec_set())
    }

    /// Access the CPU state (read-only).
    pub fn cpu(&self) -> &Cpu { &self.cpu }

    /// Access the CPU state (mutable).
    pub fn cpu_mut(&mut self) -> &mut Cpu { &mut self.cpu }

    /// Access the loaded instruction spec set.
    pub fn specs(&self) -> &InstrSpecSet { &self.specs }

    // ── Single step ───────────────────────────────────────────────────────────

    /// Fetch, decode, and execute one instruction. Returns the `InstrType` executed.
    ///
    /// The EXTEND flip-flop is cleared after each extracode instruction.
    pub fn step(&mut self) -> Result<InstrType, InterpError> {
        // 1. Fetch instruction word at Z.
        let word = self.cpu.fetch();
        let extend = self.cpu.extend;

        // 2. Store the raw instruction word in cpu for DSL access.
        self.cpu.instr_word = word;

        // 3. Decode
        let decoded = decode::decode(word, extend, &self.specs)?;
        let ty = decoded.spec.instr_type;
        let operand = decoded.address;

        // 4. Advance PC to next instruction before execution.
        self.cpu.z = self.cpu.z.wrapping_add(1) & 0x7FFF;

        // 5. Clear EXTEND flag.
        if extend {
            self.cpu.extend = false;
        }

        // 6. Execute using semantics from the spec.
        if let Some(ops) = decoded.spec.semantics.as_deref() {
            exec::exec_ops(&mut self.cpu, operand, ops)?;
        } else {
            // No semantics for this instruction
            match ty {
                InstrType::Unknown => return Err(InterpError::Halt),
                _ => return Err(InterpError::Exec(ExecError::NoSemantics(ty))),
            }
        }

        self.cycle_count += 1;
        Ok(ty)
    }

    // ── Multi-step helpers ────────────────────────────────────────────────────

    /// Run for up to `max_cycles`, returning the cycle count.
    pub fn run_for(&mut self, max_cycles: u64) -> Result<u64, InterpError> {
        for _ in 0..max_cycles {
            match self.step() {
                Ok(_)  => {}
                Err(InterpError::Halt) => return Ok(self.cycle_count),
                Err(e) => return Err(e),
            }
        }
        Ok(self.cycle_count)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_whole(opc: u16, addr: u16) -> u16 {
        ((opc & 0x7) << 12) | (addr & 0x0FFF)
    }
    fn encode_quarter(opc: u16, qtr: u16, addr: u16) -> u16 {
        ((opc & 0x1F) << 10) | ((qtr & 0x7) << 7) | (addr & 0x7F)
    }

    #[test]
    fn ca_loads_accumulator() {
        let mut interp = Interpreter::with_builtin_specs();
        interp.cpu_mut().mem_write(0o100, 99);
        let ca_instr = encode_whole(3, 0o100);
        interp.cpu_mut().mem_write(0o4000, ca_instr);
        interp.step().unwrap();
        assert_eq!(interp.cpu().a, 99);
    }

    #[test]
    fn ad_adds_to_accumulator() {
        let mut interp = Interpreter::with_builtin_specs();
        interp.cpu_mut().a = 10;
        interp.cpu_mut().mem_write(0o200, 5);
        let ad_instr = encode_whole(6, 0o200);
        interp.cpu_mut().mem_write(0o4000, ad_instr);
        interp.step().unwrap();
        assert_eq!(interp.cpu().a, 15);
    }

    #[test]
    fn tc_transfers_control() {
        let mut interp = Interpreter::with_builtin_specs();
        let tc_instr = encode_whole(0, 0o500);
        interp.cpu_mut().mem_write(0o4000, tc_instr);
        interp.step().unwrap();
        // Q should hold return address (Z+1 = 0o4001 at time of TC execution)
        assert_eq!(interp.cpu().q, 0o4001);
        // Z should be branch target
        assert_eq!(interp.cpu().z, 0o500);
    }

    #[test]
    fn extend_sets_flag() {
        let mut interp = Interpreter::with_builtin_specs();
        let ext_instr = encode_whole(0, 0o0006);
        interp.cpu_mut().mem_write(0o4000, ext_instr);
        interp.step().unwrap();
        assert!(interp.cpu().extend);
    }

    #[test]
    fn cycle_count_increments() {
        let mut interp = Interpreter::with_builtin_specs();
        for i in 0..5 {
            let ca = encode_whole(3, 0o7); // CA ZERO
            interp.cpu_mut().mem_write(0o4000 + i, ca);
        }
        for _ in 0..5 {
            interp.step().unwrap();
        }
        assert_eq!(interp.cycle_count, 5);
    }

    #[test]
    fn ccs_branches_on_positive() {
        let mut interp = Interpreter::with_builtin_specs();
        // CCS E — encode as quarter5: opc5=01, qtr=0, EXTEND required
        let ccs_instr = encode_quarter(0o01, 0, 0o50);
        interp.cpu_mut().mem_write(0o100, 0o100100); // EXTEND at 0o100
        interp.cpu_mut().mem_write(0o4000, encode_whole(0, 0o0006)); // EXTEND
        interp.cpu_mut().mem_write(0o4001, ccs_instr);
        interp.cpu_mut().mem_write(0o50, 5); // positive value at E=0o50
        // Execute EXTEND
        interp.step().unwrap();
        assert!(interp.cpu().extend);
        // Execute CCS — Z will be 0o4002 after fetch+advance
        interp.step().unwrap();
        // For positive value, pc_rel(0) = Z + 0 = 0o4002
        assert_eq!(interp.cpu().a, 5);
        assert_eq!(interp.cpu().z, 0o4002);
    }

    #[test]
    fn xch_swaps_a_and_mem() {
        let mut interp = Interpreter::with_builtin_specs();
        interp.cpu_mut().a = 0o777;
        interp.cpu_mut().mem_write(0o50, 0o123);
        // XCH E — opc5=05, qtr=5, EXTEND required
        interp.cpu_mut().mem_write(0o4000, encode_whole(0, 0o0006)); // EXTEND
        let xch = encode_quarter(0o05, 5, 0o50);
        interp.cpu_mut().mem_write(0o4001, xch);
        interp.step().unwrap(); // EXTEND
        interp.step().unwrap(); // XCH
        assert_eq!(interp.cpu().a, 0o123);
        assert_eq!(interp.cpu().mem_read(0o50), 0o777);
    }
}
