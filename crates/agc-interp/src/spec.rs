//! Instruction specification data structures for the AGC Block-2 ISA.
//!
//! These types capture all metadata needed to decode and describe an instruction.
//! They are populated by [`crate::spec_parser`] from the `ref/block2/` markdown files
//! and used by [`crate::decode`] and [`crate::interp`] to drive execution.

use std::str::FromStr;
use std::fmt;

// ─── Instruction type ────────────────────────────────────────────────────────

/// Every distinct AGC Block-2 instruction.
///
/// Variant names match the canonical AGC mnemonics (PascalCase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InstrType {
    // Sequence-changing
    Tc,    // Transfer Control to K
    Tcf,   // Transfer Control to Fixed F
    Ccs,   // Count, Compare, and Skip on E  (extracode)
    Bzf,   // Branch on Zero to Fixed F
    Bzmf,  // Branch on Zero or Minus to Fixed F

    // Fetching and storing
    Ca,    // Clear and Add K
    Cs,    // Clear and Subtract K
    Dca,   // Double Clear and Add K        (extracode)
    Dcs,   // Double Clear and Subtract K   (extracode)
    Ts,    // Transfer to Storage E         (extracode)
    Xch,   // Exchange A and E              (extracode)
    Lxch,  // Exchange L and E              (extracode)
    Qxch,  // Exchange Q and E              (extracode)
    Dxch,  // Double Exchange A and E       (extracode)

    // Modifying
    Ndx,   // Index (basic K-form or extracode E-form)

    // Arithmetic and logic
    Ad,    // Add K
    Su,    // Subtract E                    (extracode)
    Mp,    // Multiply by K                 (extracode)
    Dv,    // Divide by E                   (extracode)
    Ads,   // Add to Storage E              (extracode)
    Das,   // Double Add to Storage E       (extracode)
    Incr,  // Increment E                   (extracode)
    Aug,   // Augment E                     (extracode)
    Dim,   // Diminish E                    (extracode)
    Msu,   // Modular Subtract E            (extracode)
    Msk,   // Mask with K

    // I/O channels
    Read,  // Read channel H into A
    Write, // Write A to channel H
    Rand,  // Read channel AND A → A
    Wand,  // Write A AND channel → channel (extracode)
    Ror,   // Read channel OR A → A
    Wor,   // Write A OR channel → channel  (extracode)
    Rxor,  // Read channel XOR A → A        (extracode)

    // Special / control
    Extend, // Enable extracode for next instruction
    Inhint, // Inhibit interrupt
    Relint, // Release interrupt inhibit
    Resume, // Resume interrupted program
    Go,     // Restart at fixed address 04000

    // Involuntary — interrupt
    Rupt,   // Interrupt (saves state, vectors to handler)

    // Counter instructions (hardware-triggered, no opcode bits)
    Pinc,  // Plus Increment C
    Minc,  // Minus Increment C
    Dinc,  // Diminish Increment C
    Pcdu,  // Plus Counter Down-Up C
    Mcdu,  // Minus Counter Down-Up C
    Shinc, // Shift Increment C
    Shanc, // Shift and Add Increment C

    // Peripheral / GSE
    Tcsaj,  // Transfer Control to Specified Address K
    Fetch,  // Fetch K (display on GSE)
    Store,  // Store E (load from GSE)      (extracode)
    Inotrd, // I/O Not Read H (display on GSE)
    Inotld, // I/O Not Load H (load from GSE)

    Unknown,
}

impl InstrType {
    /// Return the canonical mnemonic string.
    pub fn mnemonic(self) -> &'static str {
        match self {
            InstrType::Tc     => "TC",
            InstrType::Tcf    => "TCF",
            InstrType::Ccs    => "CCS",
            InstrType::Bzf    => "BZF",
            InstrType::Bzmf   => "BZMF",
            InstrType::Ca     => "CA",
            InstrType::Cs     => "CS",
            InstrType::Dca    => "DCA",
            InstrType::Dcs    => "DCS",
            InstrType::Ts     => "TS",
            InstrType::Xch    => "XCH",
            InstrType::Lxch   => "LXCH",
            InstrType::Qxch   => "QXCH",
            InstrType::Dxch   => "DXCH",
            InstrType::Ndx    => "NDX",
            InstrType::Ad     => "AD",
            InstrType::Su     => "SU",
            InstrType::Mp     => "MP",
            InstrType::Dv     => "DV",
            InstrType::Ads    => "ADS",
            InstrType::Das    => "DAS",
            InstrType::Incr   => "INCR",
            InstrType::Aug    => "AUG",
            InstrType::Dim    => "DIM",
            InstrType::Msu    => "MSU",
            InstrType::Msk    => "MSK",
            InstrType::Read   => "READ",
            InstrType::Write  => "WRITE",
            InstrType::Rand   => "RAND",
            InstrType::Wand   => "WAND",
            InstrType::Ror    => "ROR",
            InstrType::Wor    => "WOR",
            InstrType::Rxor   => "RXOR",
            InstrType::Extend => "EXTEND",
            InstrType::Inhint => "INHINT",
            InstrType::Relint => "RELINT",
            InstrType::Resume => "RESUME",
            InstrType::Go     => "GO",
            InstrType::Rupt   => "RUPT",
            InstrType::Pinc   => "PINC",
            InstrType::Minc   => "MINC",
            InstrType::Dinc   => "DINC",
            InstrType::Pcdu   => "PCDU",
            InstrType::Mcdu   => "MCDU",
            InstrType::Shinc  => "SHINC",
            InstrType::Shanc  => "SHANC",
            InstrType::Tcsaj  => "TCSAJ",
            InstrType::Fetch  => "FETCH",
            InstrType::Store  => "STORE",
            InstrType::Inotrd => "INOTRD",
            InstrType::Inotld => "INOTLD",
            InstrType::Unknown => "UNKNOWN",
        }
    }

    /// True if this instruction normally requires an EXTEND prefix to decode.
    pub fn requires_extend(self) -> bool {
        matches!(self,
            InstrType::Ccs  | InstrType::Dca  | InstrType::Dcs  |
            InstrType::Ts   | InstrType::Xch  | InstrType::Lxch |
            InstrType::Qxch | InstrType::Dxch | InstrType::Su   |
            InstrType::Mp   | InstrType::Dv   | InstrType::Ads  |
            InstrType::Das  | InstrType::Incr | InstrType::Aug  |
            InstrType::Dim  | InstrType::Msu  | InstrType::Wand |
            InstrType::Wor  | InstrType::Rxor | InstrType::Store
        )
    }
}

impl FromStr for InstrType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.trim().to_uppercase().as_str() {
            "TC"     => Ok(InstrType::Tc),
            "TCF"    => Ok(InstrType::Tcf),
            "CCS"    => Ok(InstrType::Ccs),
            "BZF"    => Ok(InstrType::Bzf),
            "BZMF"   => Ok(InstrType::Bzmf),
            "CA"     => Ok(InstrType::Ca),
            "CS"     => Ok(InstrType::Cs),
            "DCA"    => Ok(InstrType::Dca),
            "DCS"    => Ok(InstrType::Dcs),
            "TS"     => Ok(InstrType::Ts),
            "XCH"    => Ok(InstrType::Xch),
            "LXCH"   => Ok(InstrType::Lxch),
            "QXCH"   => Ok(InstrType::Qxch),
            "DXCH"   => Ok(InstrType::Dxch),
            "NDX"    => Ok(InstrType::Ndx),
            "AD"     => Ok(InstrType::Ad),
            "SU"     => Ok(InstrType::Su),
            "MP"     => Ok(InstrType::Mp),
            "DV"     => Ok(InstrType::Dv),
            "ADS"    => Ok(InstrType::Ads),
            "DAS"    => Ok(InstrType::Das),
            "INCR"   => Ok(InstrType::Incr),
            "AUG"    => Ok(InstrType::Aug),
            "DIM"    => Ok(InstrType::Dim),
            "MSU"    => Ok(InstrType::Msu),
            "MSK"    => Ok(InstrType::Msk),
            "READ"   => Ok(InstrType::Read),
            "WRITE"  => Ok(InstrType::Write),
            "RAND"   => Ok(InstrType::Rand),
            "WAND"   => Ok(InstrType::Wand),
            "ROR"    => Ok(InstrType::Ror),
            "WOR"    => Ok(InstrType::Wor),
            "RXOR"   => Ok(InstrType::Rxor),
            "EXTEND" => Ok(InstrType::Extend),
            "INHINT" => Ok(InstrType::Inhint),
            "RELINT" => Ok(InstrType::Relint),
            "RESUME" => Ok(InstrType::Resume),
            "GO"     => Ok(InstrType::Go),
            "RUPT"   => Ok(InstrType::Rupt),
            "PINC"   => Ok(InstrType::Pinc),
            "MINC"   => Ok(InstrType::Minc),
            "DINC"   => Ok(InstrType::Dinc),
            "PCDU"   => Ok(InstrType::Pcdu),
            "MCDU"   => Ok(InstrType::Mcdu),
            "SHINC"  => Ok(InstrType::Shinc),
            "SHANC"  => Ok(InstrType::Shanc),
            "TCSAJ"  => Ok(InstrType::Tcsaj),
            "FETCH"  => Ok(InstrType::Fetch),
            "STORE"  => Ok(InstrType::Store),
            "INOTRD" => Ok(InstrType::Inotrd),
            "INOTLD" => Ok(InstrType::Inotld),
            _        => Err(()),
        }
    }
}

impl fmt::Display for InstrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.mnemonic())
    }
}

// ─── Address mode ─────────────────────────────────────────────────────────────

/// How the instruction addresses its operand.
///
/// From `ref/block2/OPCODE_ENCODING.md §Instruction Types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrMode {
    /// K-type — 12-bit address (CP registers 0-7, erasable 010-1777, fixed 2000-7777).
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
        match s.trim().to_uppercase().as_str() {
            "K"    => Ok(AddrMode::K),
            "E"    => Ok(AddrMode::E),
            "F"    => Ok(AddrMode::F),
            "H"    => Ok(AddrMode::H),
            "C"    => Ok(AddrMode::C),
            "NONE" | "-" | "" => Ok(AddrMode::None),
            _      => Err(()),
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
///
/// Three physical formats exist plus a virtual "hardware-triggered" category.
/// See `ref/block2/OPCODE_ENCODING.md §Instruction Word Format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeFormat {
    /// Format 1 — 3-bit opcode (bits 1-3) + 12-bit address (bits 4-15).
    /// All basic instructions; opcode values 0-7 octal.
    Whole3,

    /// Format 2 — 5-bit opcode (bits 1-5) + 3-bit quarter (bits 6-8) + 7-bit address (bits 9-15).
    /// Used for quarter-code instructions (TCF, CCS, TS, XCH, etc.).
    Quarter5,

    /// Format 3 — 6-bit opcode (bits 1-6, always 0b001000 = 010₈) + 9-bit channel address (bits 7-15).
    /// All channel I/O instructions; always require EXTEND.
    Channel6,

    /// Special fixed-word instructions (EXTEND=0o000006, INHINT=0o000004, etc.).
    Special,

    /// Counter / peripheral — no opcode bits; triggered by hardware or GSE.
    Hardware,
}

// ─── Instruction specification ────────────────────────────────────────────────

/// Complete metadata for one AGC Block-2 instruction variant.
///
/// Instances are produced by [`crate::spec_parser::SpecParser`] from the markdown
/// reference files in `ref/block2/`.  The [`crate::interp::Interpreter`] uses a
/// [`InstrSpecSet`] to decode instruction words and dispatch execution.
#[derive(Debug, Clone)]
pub struct InstrSpec {
    /// Canonical mnemonic (e.g. `"TC"`, `"CA"`, `"XCH"`).
    pub mnemonic: String,

    /// Instruction type discriminant used for dispatch.
    pub instr_type: InstrType,

    /// Operand addressing mode.
    pub addr_mode: AddrMode,

    /// Encoding format (determines how to extract opcode/address from the word).
    pub opcode_format: OpcodeFormat,

    /// Raw opcode field value (3, 5, or 6 bits depending on `opcode_format`).
    pub opcode: u8,

    /// Quarter-code field (bits 6-8) when `opcode_format == Quarter5`; `None` otherwise.
    pub quarter: Option<u8>,

    /// True if this instruction must be preceded by an EXTEND word.
    pub requires_extend: bool,

    /// Human-readable summary from the reference `.md` file.
    pub description: String,

    /// Path to the source markdown file (set when parsed from disk).
    pub source_file: Option<String>,
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
/// Built by [`crate::spec_parser::SpecParser::load`] or by
/// [`InstrSpecSet::builtin`] for embedded use.
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

    /// Look up a spec by mnemonic string (case-insensitive).
    pub fn by_mnemonic(&self, mnemonic: &str) -> Option<&InstrSpec> {
        let m = mnemonic.to_uppercase();
        self.specs.iter().find(|s| s.mnemonic.to_uppercase() == m)
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

    /// Return the built-in ISA table derived from `ref/block2/OPCODE_ENCODING.md`.
    ///
    /// This is the authoritative fallback when no markdown directory is available.
    /// Descriptions are empty; load from disk with `SpecParser::load` to enrich them.
    pub fn builtin() -> Self {
        builtin_specs()
    }
}

// ─── Built-in spec table (mirrors OPCODE_ENCODING.md) ────────────────────────

fn builtin_specs() -> InstrSpecSet {
    use OpcodeFormat::*;

    macro_rules! spec {
        ($mn:expr, $ty:expr, $mode:expr, $fmt:expr, $opc:expr, $qtr:expr, $ext:expr, $desc:expr) => {
            InstrSpec {
                mnemonic: $mn.to_string(),
                instr_type: $ty,
                addr_mode: $mode,
                opcode_format: $fmt,
                opcode: $opc,
                quarter: $qtr,
                requires_extend: $ext,
                description: $desc.to_string(),
                source_file: None,
            }
        };
    }

    let specs = vec![
        // ── Sequence changing ──────────────────────────────────────────────
        // TC  — order code 00.  (3-bit opcode 0, 12-bit address)
        spec!("TC",    InstrType::Tc,    AddrMode::K, Whole3,   0o00, None,    false, "Transfer Control to K"),
        // TCF — order codes 01.2, 01.4, 01.6 (5-bit opcode 01, quarters 2/4/6)
        spec!("TCF",   InstrType::Tcf,   AddrMode::F, Quarter5, 0o01, Some(2), false, "Transfer Control to Fixed F"),
        // CCS — order code 01.0 (5-bit opcode 01, quarter 0, EXTEND required)
        spec!("CCS",   InstrType::Ccs,   AddrMode::E, Quarter5, 0o01, Some(0), true,  "Count, Compare, and Skip on E"),
        // BZF — order codes 16.2, 16.4, 16.6
        spec!("BZF",   InstrType::Bzf,   AddrMode::F, Quarter5, 0o16, Some(2), false, "Branch on Zero to Fixed F"),
        // BZMF — order codes 12.2, 12.4, 12.6  (NOTE: shares opcode 12 with QXCH/MSU/AUG/DIM)
        spec!("BZMF",  InstrType::Bzmf,  AddrMode::F, Quarter5, 0o12, Some(2), false, "Branch on Zero or Minus to Fixed F"),

        // ── Fetching and storing ───────────────────────────────────────────
        spec!("CA",    InstrType::Ca,    AddrMode::K, Whole3,   0o03, None,    false, "Clear and Add K"),
        spec!("CS",    InstrType::Cs,    AddrMode::K, Whole3,   0o04, None,    false, "Clear and Subtract K"),
        spec!("DCA",   InstrType::Dca,   AddrMode::K, Quarter5, 0o13, None,    true,  "Double Clear and Add K"),
        spec!("DCS",   InstrType::Dcs,   AddrMode::K, Quarter5, 0o14, None,    true,  "Double Clear and Subtract K"),
        // TS  — 05.4 (EXTEND required)
        spec!("TS",    InstrType::Ts,    AddrMode::E, Quarter5, 0o05, Some(4), true,  "Transfer to Storage E"),
        // XCH — 05.5 (EXTEND required)
        spec!("XCH",   InstrType::Xch,   AddrMode::E, Quarter5, 0o05, Some(5), true,  "Exchange A and E"),
        // LXCH — 02.2 (EXTEND required)
        spec!("LXCH",  InstrType::Lxch,  AddrMode::E, Quarter5, 0o02, Some(2), true,  "Exchange L and E"),
        // QXCH — 12.2 (EXTEND required)
        spec!("QXCH",  InstrType::Qxch,  AddrMode::E, Quarter5, 0o12, Some(2), true,  "Exchange Q and E"),
        // DXCH — 05.2 (EXTEND required)
        spec!("DXCH",  InstrType::Dxch,  AddrMode::E, Quarter5, 0o05, Some(2), true,  "Double Exchange A and E"),

        // ── Modifying ──────────────────────────────────────────────────────
        // NDX basic (K): order code 05.0 without EXTEND
        // NDX extra (E): order code 05.0 with EXTEND
        // NDX whole-K:   order code 15. with EXTEND
        // Stored as one entry; decoder distinguishes by EXTEND bit.
        spec!("NDX",   InstrType::Ndx,   AddrMode::K, Quarter5, 0o05, Some(0), false, "Index next instruction"),

        // ── Arithmetic and logic ───────────────────────────────────────────
        spec!("AD",    InstrType::Ad,    AddrMode::K, Whole3,   0o06, None,    false, "Add K to A"),
        spec!("SU",    InstrType::Su,    AddrMode::E, Quarter5, 0o16, Some(0), true,  "Subtract E from A"),
        spec!("MP",    InstrType::Mp,    AddrMode::K, Quarter5, 0o17, None,    true,  "Multiply A by K"),
        spec!("DV",    InstrType::Dv,    AddrMode::E, Quarter5, 0o11, Some(0), true,  "Divide A,L by E"),
        spec!("ADS",   InstrType::Ads,   AddrMode::E, Quarter5, 0o02, Some(6), true,  "Add to Storage E"),
        spec!("DAS",   InstrType::Das,   AddrMode::E, Quarter5, 0o02, Some(0), true,  "Double Add to Storage E"),
        spec!("INCR",  InstrType::Incr,  AddrMode::E, Quarter5, 0o02, Some(4), true,  "Increment E"),
        spec!("AUG",   InstrType::Aug,   AddrMode::E, Quarter5, 0o12, Some(4), true,  "Augment E toward ±max"),
        spec!("DIM",   InstrType::Dim,   AddrMode::E, Quarter5, 0o12, Some(6), true,  "Diminish E toward zero"),
        spec!("MSU",   InstrType::Msu,   AddrMode::E, Quarter5, 0o12, Some(0), true,  "Modular Subtract E from A"),
        spec!("MSK",   InstrType::Msk,   AddrMode::K, Whole3,   0o07, None,    false, "Mask A with K"),

        // ── Channel I/O (all EXTEND required, opcode_6 = 0o10) ────────────
        // Channel variant is encoded in the top 3 bits of the 9-bit channel address.
        spec!("READ",  InstrType::Read,  AddrMode::H, Channel6, 0o10, Some(0), true,  "Read channel H into A"),
        spec!("WRITE", InstrType::Write, AddrMode::H, Channel6, 0o10, Some(1), true,  "Write A to channel H"),
        spec!("RAND",  InstrType::Rand,  AddrMode::H, Channel6, 0o10, Some(2), true,  "Read channel AND A → A"),
        spec!("WAND",  InstrType::Wand,  AddrMode::H, Channel6, 0o10, Some(3), true,  "Write A AND channel → channel"),
        spec!("ROR",   InstrType::Ror,   AddrMode::H, Channel6, 0o10, Some(4), true,  "Read channel OR A → A"),
        spec!("WOR",   InstrType::Wor,   AddrMode::H, Channel6, 0o10, Some(5), true,  "Write A OR channel → channel"),
        spec!("RXOR",  InstrType::Rxor,  AddrMode::H, Channel6, 0o10, Some(6), true,  "Read channel XOR A → A"),

        // ── Special ────────────────────────────────────────────────────────
        spec!("EXTEND", InstrType::Extend, AddrMode::None, Special, 0o00, None, false, "Enable extracode for next instruction"),
        spec!("INHINT", InstrType::Inhint, AddrMode::None, Special, 0o00, None, false, "Inhibit interrupt"),
        spec!("RELINT", InstrType::Relint, AddrMode::None, Special, 0o00, None, false, "Release interrupt inhibit"),
        spec!("RESUME", InstrType::Resume, AddrMode::None, Special, 0o05, Some(0), false, "Resume interrupted program"),
        spec!("GO",     InstrType::Go,     AddrMode::None, Special, 0o00, None, false, "Restart at 04000"),
        spec!("RUPT",   InstrType::Rupt,   AddrMode::None, Special, 0o10, None, false, "Interrupt (save state, vector to handler)"),

        // ── Counter (hardware-triggered) ───────────────────────────────────
        spec!("PINC",  InstrType::Pinc,  AddrMode::C, Hardware, 0, None, false, "Plus Increment Counter C"),
        spec!("MINC",  InstrType::Minc,  AddrMode::C, Hardware, 0, None, false, "Minus Increment Counter C"),
        spec!("DINC",  InstrType::Dinc,  AddrMode::C, Hardware, 0, None, false, "Diminish Increment Counter C"),
        spec!("PCDU",  InstrType::Pcdu,  AddrMode::C, Hardware, 0, None, false, "Plus Counter Down-Up C"),
        spec!("MCDU",  InstrType::Mcdu,  AddrMode::C, Hardware, 0, None, false, "Minus Counter Down-Up C"),
        spec!("SHINC", InstrType::Shinc, AddrMode::C, Hardware, 0, None, false, "Shift Increment C"),
        spec!("SHANC", InstrType::Shanc, AddrMode::C, Hardware, 0, None, false, "Shift and Add Increment C"),

        // ── Peripheral / GSE ───────────────────────────────────────────────
        spec!("TCSAJ",  InstrType::Tcsaj,  AddrMode::K, Whole3,   0o00, None, false, "Transfer Control to Specified Address K"),
        spec!("FETCH",  InstrType::Fetch,  AddrMode::K, Hardware, 0, None, false, "Fetch K (display on GSE)"),
        spec!("STORE",  InstrType::Store,  AddrMode::E, Hardware, 0, None, true,  "Store E (load from GSE)"),
        spec!("INOTRD", InstrType::Inotrd, AddrMode::H, Hardware, 0, None, false, "I/O Not Read H (display on GSE)"),
        spec!("INOTLD", InstrType::Inotld, AddrMode::H, Hardware, 0, None, false, "I/O Not Load H (load from GSE)"),
    ];

    InstrSpecSet::new(specs)
}
