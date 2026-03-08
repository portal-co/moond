//! High-level interpreter: fetch → decode → execute loop.
//!
//! ```no_run
//! use agc_interp::{Interpreter, InstrSpecSet};
//!
//! // Load specs from the reference docs directory
//! let specs = InstrSpecSet::builtin(); // or SpecParser::load("ref/block2/")
//! let mut interp = Interpreter::new(specs);
//!
//! // Load a program into fixed memory
//! interp.cpu_mut().mem_write(0o4000, 0o_030000); // CA 0o1000
//!
//! // Step one instruction
//! interp.step().unwrap();
//! ```

use std::collections::HashMap;

use crate::cpu::Cpu;
use crate::decode;
use crate::exec::{self, ExecError};
use crate::semantics::{self, SemOp};
use crate::spec::{InstrSpecSet, InstrType};

// ─── Interpreter state ────────────────────────────────────────────────────────

/// Interpreter error combining decode and execute failures.
#[derive(Debug)]
pub enum InterpError {
    Decode(decode::DecodeError),
    Exec(ExecError),
    /// Execution halted (e.g., GO / RESUME with no program loaded).
    Halt,
}

impl std::fmt::Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpError::Decode(e) => write!(f, "decode error: {e}"),
            InterpError::Exec(e)   => write!(f, "exec error: {e}"),
            InterpError::Halt      => f.write_str("halt"),
        }
    }
}

impl std::error::Error for InterpError {}

impl From<decode::DecodeError> for InterpError {
    fn from(e: decode::DecodeError) -> Self { InterpError::Decode(e) }
}

impl From<ExecError> for InterpError {
    fn from(e: ExecError) -> Self { InterpError::Exec(e) }
}

// ─── Interpreter ─────────────────────────────────────────────────────────────

/// The AGC Block-2 interpreter.
///
/// Holds CPU state, the instruction spec set, and a pre-compiled semantics
/// cache (maps `InstrType` → `Vec<SemOp>`).
pub struct Interpreter {
    cpu: Cpu,
    specs: InstrSpecSet,
    /// Pre-parsed semantics keyed by instruction type.
    sem_cache: HashMap<InstrType, Vec<SemOp>>,
    /// Instruction cycle counter.
    pub cycle_count: u64,
}

impl Interpreter {
    /// Create a new interpreter from an `InstrSpecSet`.
    ///
    /// Parses all built-in semantics upfront so execution is allocation-free.
    pub fn new(specs: InstrSpecSet) -> Self {
        let sem_cache = build_sem_cache();
        Interpreter {
            cpu: Cpu::new(),
            specs,
            sem_cache,
            cycle_count: 0,
        }
    }

    /// Create an interpreter using the built-in ISA table.
    pub fn with_builtin_specs() -> Self {
        Self::new(InstrSpecSet::builtin())
    }

    /// Access the CPU state (read-only).
    pub fn cpu(&self) -> &Cpu { &self.cpu }

    /// Access the CPU state (mutable, for loading programs and setting registers).
    pub fn cpu_mut(&mut self) -> &mut Cpu { &mut self.cpu }

    /// Access the loaded instruction spec set.
    pub fn specs(&self) -> &InstrSpecSet { &self.specs }

    // ── Single step ───────────────────────────────────────────────────────────

    /// Fetch, decode, and execute one instruction.  Returns the `InstrType`
    /// that was executed.
    ///
    /// The EXTEND flip-flop is cleared automatically after each extracode
    /// instruction; it is NOT cleared by EXTEND itself.
    pub fn step(&mut self) -> Result<InstrType, InterpError> {
        // 1. Fetch instruction word at Z.
        let word = self.cpu.fetch();
        let extend = self.cpu.extend;

        // 2. Decode
        let decoded = decode::decode(word, extend, &self.specs)?;
        let ty = decoded.spec.instr_type;
        let operand = decoded.address;

        // 3. Advance PC to next instruction before execution
        //    (mirrors AGC behaviour where Z+1 is available during execution)
        self.cpu.z = self.cpu.z.wrapping_add(1) & 0x7FFF;

        // 4. Clear EXTEND flag.  The flag was set by the previous EXTEND instruction.
        //    It applies to the *current* instruction; clear it now so the next
        //    instruction decodes as basic unless another EXTEND runs.
        if extend {
            self.cpu.extend = false;
        }

        // 5. Execute using the semantics from the cache.
        let ops_result: Result<(), InterpError> = if let Some(ops) = self.sem_cache.get(&ty) {
            // Data-driven execution: evaluate each SemOp in order.
            let mut pc_override: Option<u16> = None;
            let ops_clone: Vec<SemOp> = ops.clone();
            for op in &ops_clone {
                match exec::eval_op(&mut self.cpu, operand, op)? {
                    Some(new_z) => { pc_override = Some(new_z); }
                    None        => {}
                }
            }
            // Apply branch if one occurred.
            if let Some(new_z) = pc_override {
                self.cpu.z = new_z;
            }
            Ok(())
        } else {
            // Fallback: handle special cases not covered by generic SemOp.
            self.execute_fallback(ty, operand)?;
            Ok(())
        };

        ops_result?;
        self.cycle_count += 1;
        Ok(ty)
    }

    // ── Multi-step helpers ────────────────────────────────────────────────────

    /// Run until halt or error, returning the cycle count.
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

    // ── Fallback for instructions without generic semantics ───────────────────

    fn execute_fallback(&mut self, ty: InstrType, operand: u16) -> Result<(), InterpError> {
        match ty {
            InstrType::Ts => {
                exec::execute_ts(&mut self.cpu, operand);
                Ok(())
            }
            InstrType::Xch | InstrType::Lxch | InstrType::Qxch | InstrType::Dxch => {
                // Already handled in exec::execute; but in case sem_cache missed it:
                let ops = semantics::parse_sem("").unwrap();
                exec::execute(&mut self.cpu, &{
                    // Construct a minimal DecodedInstr for the fallback path
                    let spec = self.specs.by_type(ty)
                        .or_else(|| self.specs.by_type(InstrType::Unknown))
                        .unwrap();
                    decode::DecodedInstr {
                        spec,
                        address: operand,
                        raw_word: 0,
                        was_extended: false,
                    }
                }, &ops).map_err(InterpError::Exec)
            }
            InstrType::Unknown => Err(InterpError::Halt),
            _ => Err(InterpError::Exec(ExecError::NoSemantics(ty))),
        }
    }
}

// ─── Semantics cache builder ──────────────────────────────────────────────────

/// Build the `InstrType → Vec<SemOp>` lookup table from the built-in DSL strings.
fn build_sem_cache() -> HashMap<InstrType, Vec<SemOp>> {
    use InstrType::*;

    // All instruction types with their DSL strings
    let entries: &[(&str, InstrType)] = &[
        ("TC",     Tc),
        ("TCF",    Tcf),
        ("CCS",    Ccs),
        ("BZF",    Bzf),
        ("BZMF",   Bzmf),
        ("CA",     Ca),
        ("CS",     Cs),
        ("DCA",    Dca),
        ("DCS",    Dcs),
        ("TS",     Ts),
        ("XCH",    Xch),
        ("LXCH",   Lxch),
        ("QXCH",   Qxch),
        ("DXCH",   Dxch),
        ("NDX",    Ndx),
        ("AD",     Ad),
        ("SU",     Su),
        ("MP",     Mp),
        ("DV",     Dv),
        ("ADS",    Ads),
        ("DAS",    Das),
        ("INCR",   Incr),
        ("AUG",    Aug),
        ("DIM",    Dim),
        ("MSU",    Msu),
        ("MSK",    Msk),
        ("READ",   Read),
        ("WRITE",  Write),
        ("RAND",   Rand),
        ("WAND",   Wand),
        ("ROR",    Ror),
        ("WOR",    Wor),
        ("RXOR",   Rxor),
        ("EXTEND", Extend),
        ("INHINT", Inhint),
        ("RELINT", Relint),
        ("RESUME", Resume),
        ("GO",     Go),
        ("RUPT",   Rupt),
        ("PINC",   Pinc),
        ("MINC",   Minc),
        ("DINC",   Dinc),
    ];

    let mut map = HashMap::new();
    for &(mnemonic, ty) in entries {
        if let Some(ops) = semantics::builtin_sem(mnemonic) {
            map.insert(ty, ops);
        }
    }
    map
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode a 3-bit whole-code instruction word (opcode + 12-bit address).
    fn encode_whole(opc: u16, addr: u16) -> u16 {
        ((opc & 0x7) << 12) | (addr & 0x0FFF)
    }

    #[test]
    fn ca_loads_accumulator() {
        let mut interp = Interpreter::with_builtin_specs();
        // Write the value 99 to erasable address 0o100
        interp.cpu_mut().mem_write(0o100, 99);
        // Place CA 0o100 at the start address (Z = 0o4000)
        // CA = opcode 3
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
        // AD = opcode 6
        let ad_instr = encode_whole(6, 0o200);
        interp.cpu_mut().mem_write(0o4000, ad_instr);
        interp.step().unwrap();
        assert_eq!(interp.cpu().a, 15);
    }

    #[test]
    fn tc_transfers_control() {
        let mut interp = Interpreter::with_builtin_specs();
        // TC 0o500 — TC = opcode 0
        let tc_instr = encode_whole(0, 0o500);
        interp.cpu_mut().mem_write(0o4000, tc_instr);
        interp.step().unwrap();
        // Q should hold the return address (0o4001 — next after TC)
        assert_eq!(interp.cpu().q, 0o4001);
        // Z should be the branch target + 1 (we set Z to target then advance)
        // Actually: Z is set to operand (0o500) during branch, then Z is
        // the "next fetch" address. Since TC sets Z = operand, next fetch is at 0o500.
        assert_eq!(interp.cpu().z, 0o500);
    }

    #[test]
    fn extend_sets_flag() {
        let mut interp = Interpreter::with_builtin_specs();
        // EXTEND = TC 0o0006 = encode_whole(0, 6)
        let ext_instr = encode_whole(0, 0o0006);
        interp.cpu_mut().mem_write(0o4000, ext_instr);
        interp.step().unwrap();
        assert!(interp.cpu().extend);
    }

    #[test]
    fn cycle_count_increments() {
        let mut interp = Interpreter::with_builtin_specs();
        // NOP-like: CA 0 (zero register) several times
        for i in 0..5 {
            let ca = encode_whole(3, 0o7); // CA ZERO
            interp.cpu_mut().mem_write(0o4000 + i, ca);
        }
        for _ in 0..5 {
            interp.step().unwrap();
        }
        assert_eq!(interp.cycle_count, 5);
    }
}
