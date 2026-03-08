//! Decode a 15-bit AGC instruction word into a [`DecodedInstr`].
//!
//! The decoding logic mirrors `src/decode.c` and the tables in
//! `ref/block2/OPCODE_ENCODING.md`.
//!
//! # Bit layout (AGC vs Rust `u16`)
//!
//! AGC bit numbering is **backwards**: AGC bit 1 is the MSB of the 15-bit data field,
//! AGC bit 15 is the LSB.  After stripping the parity bit (bit 0), we store the
//! 15-bit instruction word in a `u16` where:
//!
//! ```text
//! Rust bit index: 14  13  12  11  10   9   8   7   6   5   4   3   2   1   0
//! AGC bit label:   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
//! ```
//!
//! Field extraction (all as Rust-native bit indices, MSB at index 14):
//!
//! | Field      | AGC bits | Rust bit range | Extract                         |
//! |------------|----------|----------------|---------------------------------|
//! | opcode_3   | 1–3      | 14–12          | `(w >> 12) & 0x7`               |
//! | opcode_5   | 1–5      | 14–10          | `(w >> 10) & 0x1F`              |
//! | opcode_6   | 1–6      | 14–9           | `(w >> 9) & 0x3F`               |
//! | quarter    | 6–8      | 9–7            | `(w >> 7) & 0x7`                |
//! | addr_12    | 4–15     | 11–0           | `w & 0x0FFF`                    |
//! | addr_9     | 7–15     | 8–0            | `w & 0x01FF`                    |
//! | addr_7     | 9–15     | 6–0            | `w & 0x007F`                    |

use crate::spec::{InstrSpec, InstrSpecSet, InstrType};

// ─── Decoded instruction ──────────────────────────────────────────────────────

/// Result of decoding a single 15-bit instruction word.
#[derive(Debug, Clone)]
pub struct DecodedInstr<'s> {
    /// Resolved instruction spec (points into the `InstrSpecSet`).
    pub spec: &'s InstrSpec,
    /// Effective operand address extracted from the word.
    pub address: u16,
    /// The raw 15-bit instruction word.
    pub raw_word: u16,
    /// Whether the EXTEND flip-flop was set when this was decoded.
    pub was_extended: bool,
}

/// Decode error (unknown or illegal encoding).
#[derive(Debug, Clone)]
pub struct DecodeError {
    pub word: u16,
    pub extend: bool,
    pub message: &'static str,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "decode error (word={:#07o}, extend={}) — {}",
               self.word, self.extend, self.message)
    }
}

impl std::error::Error for DecodeError {}

// ─── Field extraction helpers ─────────────────────────────────────────────────

/// 3-bit whole-code opcode (AGC bits 1–3 → Rust bits 14–12).
#[inline]
fn opcode_3(w: u16) -> u8 { ((w >> 12) & 0x7) as u8 }

/// 5-bit quarter-code opcode (AGC bits 1–5 → Rust bits 14–10).
#[inline]
fn opcode_5(w: u16) -> u8 { ((w >> 10) & 0x1F) as u8 }

/// 6-bit channel opcode (AGC bits 1–6 → Rust bits 14–9).
#[inline]
fn opcode_6(w: u16) -> u8 { ((w >> 9) & 0x3F) as u8 }

/// Quarter field (AGC bits 6–8 → Rust bits 9–7).
#[inline]
fn quarter(w: u16) -> u8 { ((w >> 7) & 0x7) as u8 }

/// 12-bit address (AGC bits 4–15 → Rust bits 11–0).
#[inline]
fn addr_12(w: u16) -> u16 { w & 0x0FFF }

/// 9-bit channel address (AGC bits 7–15 → Rust bits 8–0).
#[inline]
fn addr_9(w: u16) -> u16 { w & 0x01FF }

/// 7-bit address for quarter-code instructions (AGC bits 9–15 → Rust bits 6–0).
#[inline]
fn addr_7(w: u16) -> u16 { w & 0x007F }

// ─── Decoder ─────────────────────────────────────────────────────────────────

/// Decode a 15-bit instruction word using the given `InstrSpecSet`.
///
/// `extend` should be `true` when the EXTEND flip-flop is set.
pub fn decode<'s>(
    word: u16,
    extend: bool,
    specs: &'s InstrSpecSet,
) -> Result<DecodedInstr<'s>, DecodeError> {
    let word = word & 0x7FFF; // strip any stray parity bit

    // ── Section 1: Channel instructions (opcode_6 == 0o10 = 8) ─────────────
    // All channel instructions require EXTEND.  They use a 6-bit opcode field
    // (bits 1–6) of 0b001000 followed by a 9-bit channel address.
    if opcode_6(word) == 0o10 && extend {
        let ch_addr  = addr_9(word);
        let variant  = ((ch_addr >> 6) & 0x7) as u8; // top 3 bits of 9-bit addr
        let ch_addr  = ch_addr & 0x01FF;

        let ty = match variant {
            0 => InstrType::Read,
            1 => InstrType::Write,
            2 => InstrType::Rand,
            3 => InstrType::Wand,
            4 => InstrType::Ror,
            5 => InstrType::Wor,
            6 => InstrType::Rxor,
            _ => return err(word, extend, "unknown channel variant"),
        };
        return lookup(specs, ty, ch_addr, word, extend);
    }

    // ── Section 2: Quarter-code instructions (opcode_5 with quarter field) ──
    // These occupy opcodes 0o01 – 0o17 (octal) in the 5-bit field.
    let opc5 = opcode_5(word);
    let qtr  = quarter(word);
    let a7   = addr_7(word);

    // Extended (EXTEND=1) quarter-code instructions
    if extend {
        match (opc5, qtr) {
            // CCS E — 01.0 with EXTEND
            (0o01, 0) => return lookup(specs, InstrType::Ccs,  a7, word, extend),
            // DAS E — 02.0 with EXTEND
            (0o02, 0) => return lookup(specs, InstrType::Das,  a7, word, extend),
            // LXCH E — 02.2 with EXTEND
            (0o02, 2) => return lookup(specs, InstrType::Lxch, a7, word, extend),
            // INCR E — 02.4 with EXTEND
            (0o02, 4) => return lookup(specs, InstrType::Incr, a7, word, extend),
            // ADS E — 02.6 with EXTEND
            (0o02, 6) => return lookup(specs, InstrType::Ads,  a7, word, extend),
            // DXCH E — 05.2 with EXTEND
            (0o05, 2) => return lookup(specs, InstrType::Dxch, a7, word, extend),
            // TS E — 05.4 with EXTEND
            (0o05, 4) => return lookup(specs, InstrType::Ts,   a7, word, extend),
            // XCH E — 05.5 with EXTEND
            (0o05, 5) => return lookup(specs, InstrType::Xch,  a7, word, extend),
            // NDX E — 05.0 with EXTEND
            (0o05, 0) => return lookup(specs, InstrType::Ndx,  a7, word, extend),
            // DV E — 11.0 with EXTEND
            (0o11, 0) => return lookup(specs, InstrType::Dv,   a7, word, extend),
            // MSU E — 12.0 with EXTEND
            (0o12, 0) => return lookup(specs, InstrType::Msu,  a7, word, extend),
            // QXCH E — 12.2 with EXTEND
            (0o12, 2) => return lookup(specs, InstrType::Qxch, a7, word, extend),
            // AUG E — 12.4 with EXTEND
            (0o12, 4) => return lookup(specs, InstrType::Aug,  a7, word, extend),
            // DIM E — 12.6 with EXTEND
            (0o12, 6) => return lookup(specs, InstrType::Dim,  a7, word, extend),
            // DCA K — 13. with EXTEND (all quarters)
            (0o13, _) => return lookup(specs, InstrType::Dca, addr_12(word), word, extend),
            // DCS K — 14. with EXTEND (all quarters)
            (0o14, _) => return lookup(specs, InstrType::Dcs, addr_12(word), word, extend),
            // NDX K — 15. with EXTEND (all quarters)
            (0o15, _) => return lookup(specs, InstrType::Ndx, addr_12(word), word, extend),
            // BZF F — 16.2, 16.4, 16.6 with EXTEND
            (0o16, 2) | (0o16, 4) | (0o16, 6) => return lookup(specs, InstrType::Bzf, a7, word, extend),
            // SU E — 16.0 with EXTEND
            (0o16, 0) => return lookup(specs, InstrType::Su, a7, word, extend),
            // MP K — 17. with EXTEND (all quarters)
            (0o17, _) => return lookup(specs, InstrType::Mp, addr_12(word), word, extend),
            // BZMF F — 12.2, 12.4, 12.6 with EXTEND
            // Note: BZMF and QXCH share opcode 12.2; BZMF requires EXTEND per spec table.
            // However the OPCODE_ENCODING table lists BZMF without EXTEND for some quarters.
            // We treat 12.{2,4,6} without EXTEND as BZF/BZMF (branch variants).
            _ => {}  // fall through to basic decode for unhandled extended opcodes
        }
    }

    // Non-extended quarter-code instructions
    match (opc5, qtr) {
        // TCF F — 01.{2,4,6}
        (0o01, 2) | (0o01, 4) | (0o01, 6) => return lookup(specs, InstrType::Tcf, a7, word, extend),
        // LXCH E — basic form 02.{0,2,4,6} without EXTEND
        (0o02, 0) | (0o02, 2) | (0o02, 4) | (0o02, 6) => return lookup(specs, InstrType::Lxch, a7, word, extend),
        // DXCH — 05.2 basic
        (0o05, 2) => return lookup(specs, InstrType::Dxch, a7, word, extend),
        // TS — 05.4 basic
        (0o05, 4) => return lookup(specs, InstrType::Ts,   a7, word, extend),
        // XCH — 05.5 basic
        (0o05, 5) => return lookup(specs, InstrType::Xch,  a7, word, extend),
        // NDX K — 05.0 basic
        (0o05, 0) => return lookup(specs, InstrType::Ndx,  addr_12(word), word, extend),
        // QXCH — 12.{0,2,4,6} basic
        (0o12, 0) | (0o12, 2) | (0o12, 4) | (0o12, 6) => return lookup(specs, InstrType::Qxch, a7, word, extend),
        // BZMF F — 12.{2,4,6} basic (some share with QXCH; actual hardware differs)
        // We treat as QXCH when not extended; BZMF requires EXTEND per the encoding doc.
        _ => {}
    }

    // ── Section 3: Whole-code instructions (3-bit opcode, 12-bit address) ───
    let opc3 = opcode_3(word);
    let a12  = addr_12(word);

    // Check for special fixed-word instructions first (TC with specific low addresses).
    if opc3 == 0o0 {
        // TC opcode 0: special cases encoded in address field
        match a12 {
            0o0006 => return lookup(specs, InstrType::Extend, 0, word, extend),
            0o0004 => return lookup(specs, InstrType::Inhint, 0, word, extend),
            0o0003 => return lookup(specs, InstrType::Relint, 0, word, extend),
            0o4000 => return lookup(specs, InstrType::Go,     0, word, extend),
            _      => {
                // Regular TC K
                return lookup(specs, InstrType::Tc, a12, word, extend);
            }
        }
    }

    // RESUME — special: 05.0017 without EXTEND
    if opc5 == 0o05 && a7 == 0o017 && !extend {
        return lookup(specs, InstrType::Resume, 0, word, extend);
    }

    // RUPT — opcode 0o10 (whole-code) without EXTEND
    if opc3 == 0o10 && !extend {
        return lookup(specs, InstrType::Rupt, 0, word, extend);
    }

    match opc3 {
        0o1 => return lookup(specs, InstrType::Tc,  a12, word, extend),  // TC K (alternate)
        0o2 => return lookup(specs, InstrType::Tc,  a12, word, extend),  // TC K (alternate)
        0o3 => return lookup(specs, InstrType::Ca,  a12, word, extend),  // CA K
        0o4 => return lookup(specs, InstrType::Cs,  a12, word, extend),  // CS K
        0o5 => return lookup(specs, InstrType::Tc,  a12, word, extend),  // TC K (TS/XCH handled above)
        0o6 => return lookup(specs, InstrType::Ad,  a12, word, extend),  // AD K
        0o7 => return lookup(specs, InstrType::Msk, a12, word, extend),  // MSK K
        _   => {}
    }

    err(word, extend, "no matching instruction encoding")
}

fn lookup<'s>(
    specs: &'s InstrSpecSet,
    ty: InstrType,
    address: u16,
    raw_word: u16,
    extend: bool,
) -> Result<DecodedInstr<'s>, DecodeError> {
    // Find the spec; use the Unknown spec as a fallback so callers always get a DecodedInstr.
    let spec = specs.by_type(ty)
        .or_else(|| specs.by_type(InstrType::Unknown))
        .ok_or_else(|| DecodeError { word: raw_word, extend, message: "spec not found" })?;
    Ok(DecodedInstr { spec, address, raw_word, was_extended: extend })
}

fn err(word: u16, extend: bool, message: &'static str) -> Result<DecodedInstr<'static>, DecodeError> {
    Err(DecodeError { word, extend, message })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::InstrSpecSet;

    fn specs() -> InstrSpecSet { InstrSpecSet::builtin() }

    /// Helper: encode a whole-code instruction (3-bit opc, 12-bit addr).
    fn encode_whole(opc: u16, addr: u16) -> u16 { (opc << 12) | (addr & 0x0FFF) }
    /// Helper: encode a quarter-code instruction (5-bit opc, 3-bit qtr, 7-bit addr).
    fn encode_quarter(opc: u16, qtr: u16, addr: u16) -> u16 {
        ((opc & 0x1F) << 10) | ((qtr & 0x7) << 7) | (addr & 0x7F)
    }

    #[test]
    fn decode_tc() {
        let s = specs();
        // TC K=0o100 → opc3=0, addr=0o100
        let w = encode_whole(0o0, 0o100);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Tc);
        assert_eq!(d.address, 0o100);
    }

    #[test]
    fn decode_ca() {
        let s = specs();
        let w = encode_whole(0o3, 0o200);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Ca);
        assert_eq!(d.address, 0o200);
    }

    #[test]
    fn decode_ad() {
        let s = specs();
        let w = encode_whole(0o6, 0o150);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Ad);
    }

    #[test]
    fn decode_extend() {
        let s = specs();
        // EXTEND = TC 0o0006 = whole-code opc=0, addr=6
        let w = encode_whole(0o0, 0o0006);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Extend);
    }

    #[test]
    fn decode_ccs_requires_extend() {
        let s = specs();
        // CCS E = 01.0 with EXTEND → opc5=1, qtr=0
        let w = encode_quarter(0o01, 0, 0o050);
        let d = decode(w, true, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Ccs);
        assert_eq!(d.address, 0o050);
    }

    #[test]
    fn decode_tcf() {
        let s = specs();
        // TCF F = 01.2 → opc5=1, qtr=2, no extend needed
        let w = encode_quarter(0o01, 2, 0o020);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Tcf);
    }

    #[test]
    fn decode_read_channel() {
        let s = specs();
        // READ H=030 — opcode_6=010(octal), variant=0, channel=030
        // Bit layout: bits14..9 = 001000, bits8..0 = 0_000_011_000
        // opcode_6(word) = (word >> 9) & 0x3F = 0b001000 = 8 = 010 octal
        // channel addr = word & 0x01FF = 030 octal = 24
        let w: u16 = (0b001000 << 9) | 0o030; // READ + channel 030
        let d = decode(w, true, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Read);
        assert_eq!(d.address, 0o030);
    }
}
