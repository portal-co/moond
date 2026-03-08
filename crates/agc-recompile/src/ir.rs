//! IR types for the AGC recompiler frontend output.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use agc_isa::{InstrType, SemOp};

// ─── Instruction record ───────────────────────────────────────────────────────

/// One decoded AGC instruction with its lowered TC2 bytecode.
#[derive(Debug, Clone)]
pub struct InstrRecord {
    /// AGC address of this instruction (12-bit, 0o0000–0o7777).
    pub pc: u16,
    /// Raw 15-bit instruction word.
    pub raw_word: u16,
    /// Whether the EXTEND flip-flop was set when this instruction was decoded.
    pub was_extended: bool,
    /// Decoded instruction type.
    pub instr_type: InstrType,
    /// Canonical mnemonic (e.g. `"CA"`, `"TCF"`).
    pub mnemonic: String,
    /// Effective operand / address field extracted from the word.
    pub operand: u16,
    /// Semantic operations (from the ISA spec).
    pub ops: Vec<SemOp>,
    /// Lowered TC2 16-bit stack bytecode.
    pub bytecode: Vec<u16>,
}

// ─── Terminator ───────────────────────────────────────────────────────────────

/// How control leaves a basic block.
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Falls through to the immediately following address.
    FallThrough(u16),
    /// Unconditional direct jump.
    Jump(u16),
    /// Single-condition direct branch: if condition holds → `taken`, else → `fallthru`.
    CondBranch { taken: u16, fallthru: u16 },
    /// CCS 4-way branch: positive → `[0]`, plus-zero → `[1]`, negative → `[2]`, minus-zero → `[3]`.
    CcsBranch([u16; 4]),
    /// Indirect / dynamic dispatch.  The `possible_targets` list comes from
    /// the caller-supplied indirect-target set plus any statically resolvable
    /// fixed-memory reads that the frontend could determine.
    Indirect { possible_targets: Vec<u16> },
    /// Unreachable / halt (used for unknown instructions).
    Halt,
}

impl Terminator {
    /// All statically-known successor addresses.
    pub fn successors(&self) -> alloc::vec::IntoIter<u16> {
        let v: Vec<u16> = match self {
            Terminator::FallThrough(a)          => alloc::vec![*a],
            Terminator::Jump(a)                 => alloc::vec![*a],
            Terminator::CondBranch { taken, fallthru } => alloc::vec![*taken, *fallthru],
            Terminator::CcsBranch(t)            => t.to_vec(),
            Terminator::Indirect { possible_targets } => possible_targets.clone(),
            Terminator::Halt                    => alloc::vec![],
        };
        v.into_iter()
    }
}

// ─── Basic block ──────────────────────────────────────────────────────────────

/// A maximal sequence of instructions with a single entry and a known exit.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// AGC address of the first instruction in this block.
    pub label: u16,
    /// Decoded instructions in execution order.
    pub instrs: Vec<InstrRecord>,
    /// How control leaves this block.
    pub terminator: Terminator,
}

// ─── Instruction stream ───────────────────────────────────────────────────────

/// The complete reachable IR produced by the frontend.
#[derive(Debug)]
pub struct InstrStream {
    /// All discovered basic blocks keyed by their entry address.
    pub blocks: BTreeMap<u16, BasicBlock>,
    /// Addresses from which decode was seeded (original entry points).
    pub entry_points: Vec<u16>,
    /// Full set of indirect branch targets supplied by the caller.
    pub indirect_targets: BTreeSet<u16>,
}
