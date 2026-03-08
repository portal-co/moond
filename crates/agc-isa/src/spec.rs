//! Instruction specification data structures for the AGC Block-2 ISA.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::instr_type::InstrType;
use crate::semantics::SemOp;

// ─── Address mode ─────────────────────────────────────────────────────────────

/// How the instruction addresses its operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrMode {
    /// K-type — 12-bit address.
    K,
    /// E-type — 12-bit erasable-memory address.
    E,
    /// F-type — 12-bit fixed-memory address (branch targets).
    F,
    /// H-type — 9-bit I/O channel address.
    H,
    /// C-type — counter address (determined by hardware, no opcode bits).
    C,
    /// No operand address.
    None,
}

impl FromStr for AddrMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        // Manual uppercase comparison (no_std friendly)
        let s = s.trim();
        match s {
            "K" | "k"    => Ok(AddrMode::K),
            "E" | "e"    => Ok(AddrMode::E),
            "F" | "f"    => Ok(AddrMode::F),
            "H" | "h"    => Ok(AddrMode::H),
            "C" | "c"    => Ok(AddrMode::C),
            "NONE" | "none" | "-" | "" => Ok(AddrMode::None),
            _            => Err(()),
        }
    }
}

impl fmt::Display for AddrMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddrMode::K    => f.write_str("K"),
            AddrMode::E    => f.write_str("E"),
            AddrMode::F    => f.write_str("F"),
            AddrMode::H    => f.write_str("H"),
            AddrMode::C    => f.write_str("C"),
            AddrMode::None => f.write_str("-"),
        }
    }
}

// ─── Opcode encoding format ───────────────────────────────────────────────────

/// Bit-layout format of an instruction word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeFormat {
    /// Format 1 — 3-bit opcode + 12-bit address.
    Whole3,
    /// Format 2 — 5-bit opcode + 3-bit quarter + 7-bit address.
    Quarter5,
    /// Format 3 — 6-bit opcode (always 0o010) + 9-bit channel address.
    Channel6,
    /// Special fixed-word instructions.
    Special,
    /// Counter / peripheral — no opcode bits; triggered by hardware or GSE.
    Hardware,
}

// ─── Instruction specification ────────────────────────────────────────────────

/// Complete metadata for one AGC Block-2 instruction variant.
#[derive(Debug, Clone)]
pub struct InstrSpec {
    /// Canonical mnemonic (e.g. `"TC"`, `"CA"`, `"XCH"`).
    pub mnemonic: String,
    /// Instruction type discriminant used for dispatch.
    pub instr_type: InstrType,
    /// Operand addressing mode.
    pub addr_mode: AddrMode,
    /// Encoding format.
    pub opcode_format: OpcodeFormat,
    /// Raw opcode field value.
    pub opcode: u8,
    /// Quarter-code field when `opcode_format == Quarter5`; `None` otherwise.
    pub quarter: Option<u8>,
    /// True if this instruction must be preceded by an EXTEND word.
    pub requires_extend: bool,
    /// Human-readable summary.
    pub description: String,
    /// Path to the source markdown file (set when parsed from disk).
    pub source_file: Option<String>,
    /// Parsed semantics (populated by builtin or DSL loader).
    pub semantics: Option<Vec<SemOp>>,
}

impl InstrSpec {
    /// Construct a minimal spec with defaults.
    pub fn new(mnemonic: impl Into<String>, instr_type: InstrType) -> Self {
        let requires_extend = instr_type.requires_extend();
        InstrSpec {
            mnemonic: mnemonic.into(),
            instr_type,
            addr_mode: AddrMode::None,
            opcode_format: OpcodeFormat::Hardware,
            opcode: 0,
            quarter: None,
            requires_extend,
            description: String::new(),
            source_file: None,
            semantics: None,
        }
    }
}

impl fmt::Display for InstrSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.mnemonic, self.addr_mode)?;
        if self.requires_extend { f.write_str(" [EXT]")?; }
        if !self.description.is_empty() {
            write!(f, " — {}", self.description)?;
        }
        Ok(())
    }
}

// ─── Instruction specification set ───────────────────────────────────────────

/// A complete collection of `InstrSpec` values for the Block-2 ISA.
///
/// Uses `Vec<InstrSpec>` with linear search (no HashMap — no_std compatible).
pub struct InstrSpecSet {
    specs: Vec<InstrSpec>,
}

impl InstrSpecSet {
    /// Create from a pre-built vector of specs.
    pub fn new(specs: Vec<InstrSpec>) -> Self {
        InstrSpecSet { specs }
    }

    /// Look up a spec by `InstrType`.
    pub fn by_type(&self, instr_type: InstrType) -> Option<&InstrSpec> {
        self.specs.iter().find(|s| s.instr_type == instr_type)
    }

    /// Look up a spec (mutable) by `InstrType`.
    pub fn by_type_mut(&mut self, instr_type: InstrType) -> Option<&mut InstrSpec> {
        self.specs.iter_mut().find(|s| s.instr_type == instr_type)
    }

    /// Look up a spec by mnemonic string (case-insensitive ASCII).
    pub fn by_mnemonic(&self, mnemonic: &str) -> Option<&InstrSpec> {
        self.specs.iter().find(|s| s.mnemonic.eq_ignore_ascii_case(mnemonic))
    }

    /// Iterate over all specs.
    pub fn iter(&self) -> impl Iterator<Item = &InstrSpec> {
        self.specs.iter()
    }

    /// Number of specs loaded.
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// Add or replace a spec (matched by `InstrType`).
    pub fn insert(&mut self, spec: InstrSpec) {
        if let Some(existing) = self.specs.iter_mut().find(|s| s.instr_type == spec.instr_type) {
            *existing = spec;
        } else {
            self.specs.push(spec);
        }
    }
}
