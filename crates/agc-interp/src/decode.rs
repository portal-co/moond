//! Decode a 15-bit AGC instruction word into a [`DecodedInstr`].

use core::fmt;
use agc_isa::spec::{InstrSpec, InstrSpecSet};
use agc_isa::instr_type::InstrType;

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

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "decode error (word={:#07o}, extend={}) — {}",
               self.word, self.extend, self.message)
    }
}

// ─── Field extraction helpers ─────────────────────────────────────────────────

#[inline]
fn opcode_3(w: u16) -> u8 { ((w >> 12) & 0x7) as u8 }
#[inline]
fn opcode_5(w: u16) -> u8 { ((w >> 10) & 0x1F) as u8 }
#[inline]
fn opcode_6(w: u16) -> u8 { ((w >> 9) & 0x3F) as u8 }
#[inline]
fn quarter(w: u16) -> u8 { ((w >> 7) & 0x7) as u8 }
#[inline]
fn addr_12(w: u16) -> u16 { w & 0x0FFF }
#[inline]
fn addr_9(w: u16) -> u16 { w & 0x01FF }
#[inline]
fn addr_7(w: u16) -> u16 { w & 0x007F }

// ─── Decoder ─────────────────────────────────────────────────────────────────

/// Decode a 15-bit instruction word using the given `InstrSpecSet`.
pub fn decode<'s>(
    word: u16,
    extend: bool,
    specs: &'s InstrSpecSet,
) -> Result<DecodedInstr<'s>, DecodeError> {
    let word = word & 0x7FFF;

    // Section 1: Channel instructions (opcode_6 == 0o10 = 8)
    if opcode_6(word) == 0o10 && extend {
        let ch_addr  = addr_9(word);
        let variant  = ((ch_addr >> 6) & 0x7) as u8;
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

    // Section 2: Quarter-code instructions
    let opc5 = opcode_5(word);
    let qtr  = quarter(word);
    let a7   = addr_7(word);

    // Extended quarter-code instructions
    if extend {
        match (opc5, qtr) {
            (0o01, 0) => return lookup(specs, InstrType::Ccs,  a7, word, extend),
            (0o02, 0) => return lookup(specs, InstrType::Das,  a7, word, extend),
            (0o02, 2) => return lookup(specs, InstrType::Lxch, a7, word, extend),
            (0o02, 4) => return lookup(specs, InstrType::Incr, a7, word, extend),
            (0o02, 6) => return lookup(specs, InstrType::Ads,  a7, word, extend),
            (0o05, 2) => return lookup(specs, InstrType::Dxch, a7, word, extend),
            (0o05, 4) => return lookup(specs, InstrType::Ts,   a7, word, extend),
            (0o05, 5) => return lookup(specs, InstrType::Xch,  a7, word, extend),
            (0o05, 0) => return lookup(specs, InstrType::Ndx,  a7, word, extend),
            (0o11, 0) => return lookup(specs, InstrType::Dv,   a7, word, extend),
            (0o12, 0) => return lookup(specs, InstrType::Msu,  a7, word, extend),
            (0o12, 2) => return lookup(specs, InstrType::Qxch, a7, word, extend),
            (0o12, 4) => return lookup(specs, InstrType::Aug,  a7, word, extend),
            (0o12, 6) => return lookup(specs, InstrType::Dim,  a7, word, extend),
            (0o13, _) => return lookup(specs, InstrType::Dca, addr_12(word), word, extend),
            (0o14, _) => return lookup(specs, InstrType::Dcs, addr_12(word), word, extend),
            (0o15, _) => return lookup(specs, InstrType::Ndx, addr_12(word), word, extend),
            (0o16, 2) | (0o16, 4) | (0o16, 6) => return lookup(specs, InstrType::Bzf, a7, word, extend),
            (0o16, 0) => return lookup(specs, InstrType::Su, a7, word, extend),
            (0o17, _) => return lookup(specs, InstrType::Mp, addr_12(word), word, extend),
            _ => {}
        }
    }

    // Non-extended quarter-code instructions
    match (opc5, qtr) {
        (0o01, 2) | (0o01, 4) | (0o01, 6) => return lookup(specs, InstrType::Tcf, a7, word, extend),
        (0o02, 0) | (0o02, 2) | (0o02, 4) | (0o02, 6) => return lookup(specs, InstrType::Lxch, a7, word, extend),
        (0o05, 2) => return lookup(specs, InstrType::Dxch, a7, word, extend),
        (0o05, 4) => return lookup(specs, InstrType::Ts,   a7, word, extend),
        (0o05, 5) => return lookup(specs, InstrType::Xch,  a7, word, extend),
        (0o05, 0) => return lookup(specs, InstrType::Ndx,  addr_12(word), word, extend),
        (0o12, 0) | (0o12, 2) | (0o12, 4) | (0o12, 6) => return lookup(specs, InstrType::Qxch, a7, word, extend),
        _ => {}
    }

    // Section 3: Whole-code instructions
    let opc3 = opcode_3(word);
    let a12  = addr_12(word);

    if opc3 == 0o0 {
        match a12 {
            0o0006 => return lookup(specs, InstrType::Extend, 0, word, extend),
            0o0004 => return lookup(specs, InstrType::Inhint, 0, word, extend),
            0o0003 => return lookup(specs, InstrType::Relint, 0, word, extend),
            0o4000 => return lookup(specs, InstrType::Go,     0, word, extend),
            _      => return lookup(specs, InstrType::Tc, a12, word, extend),
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
        0o1 => return lookup(specs, InstrType::Tc,  a12, word, extend),
        0o2 => return lookup(specs, InstrType::Tc,  a12, word, extend),
        0o3 => return lookup(specs, InstrType::Ca,  a12, word, extend),
        0o4 => return lookup(specs, InstrType::Cs,  a12, word, extend),
        0o5 => return lookup(specs, InstrType::Tc,  a12, word, extend),
        0o6 => return lookup(specs, InstrType::Ad,  a12, word, extend),
        0o7 => return lookup(specs, InstrType::Msk, a12, word, extend),
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
    use agc_isa::builtin_spec_set;

    fn specs() -> InstrSpecSet { builtin_spec_set() }

    fn encode_whole(opc: u16, addr: u16) -> u16 { (opc << 12) | (addr & 0x0FFF) }
    fn encode_quarter(opc: u16, qtr: u16, addr: u16) -> u16 {
        ((opc & 0x1F) << 10) | ((qtr & 0x7) << 7) | (addr & 0x7F)
    }

    #[test]
    fn decode_tc() {
        let s = specs();
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
        let w = encode_whole(0o0, 0o0006);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Extend);
    }

    #[test]
    fn decode_ccs_requires_extend() {
        let s = specs();
        let w = encode_quarter(0o01, 0, 0o050);
        let d = decode(w, true, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Ccs);
        assert_eq!(d.address, 0o050);
    }

    #[test]
    fn decode_tcf() {
        let s = specs();
        let w = encode_quarter(0o01, 2, 0o020);
        let d = decode(w, false, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Tcf);
    }

    #[test]
    fn decode_read_channel() {
        let s = specs();
        let w: u16 = (0b001000 << 9) | 0o030;
        let d = decode(w, true, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Read);
        assert_eq!(d.address, 0o030);
    }
}
