//! Assembler for the AGC Block-2 ISA.

use agc_isa::{builtin_spec_set, InstrType, OpcodeFormat};

/// The raw 15-bit word for the EXTEND instruction (TC 0o0006 = 6).
const EXTEND_WORD: u16 = 0o000006;

/// Encode a single instruction word from its spec and user-supplied address.
///
/// Returns an error if the instruction format is `Hardware` (not user-assemblable).
fn encode_word(spec: &agc_isa::InstrSpec, addr: u16) -> Result<u16, String> {
    let word = match spec.opcode_format {
        OpcodeFormat::Whole3 => {
            (spec.opcode as u16) << 12 | (addr & 0x0FFF)
        }
        OpcodeFormat::Quarter5 => {
            // NDX uses a 10-bit address field for addresses > 0o177 (7-bit max):
            //   opcode 0o15 with bits [9:7] embedded in the quarter position.
            if spec.instr_type == InstrType::Ndx && addr > 0o177 {
                (0o15u16 << 10) | (addr & 0x03FF)
            } else {
                (spec.opcode as u16) << 10
                    | ((spec.quarter.unwrap_or(0) as u16) << 7)
                    | (addr & 0x007F)
            }
        }
        OpcodeFormat::Channel6 => {
            // Channel instructions: opc6=0o10, then bits [8:6] of addr9 hold
            // the channel variant (quarter field), and bits [5:0] the channel number.
            let variant = spec.quarter.unwrap_or(0) as u16;
            let ch = addr & 0x01FF; // keep full 9-bit channel address
            // The decoder reads variant from (ch_addr >> 6) & 0x7 where ch_addr = addr_9(word).
            // So we must embed variant into the upper bits of the 9-bit field.
            (spec.opcode as u16) << 9 | (variant << 6) | (ch & 0x3F)
        }
        OpcodeFormat::Special => {
            // Fixed-address instructions; address is determined by InstrType.
            let fixed_addr: u16 = match spec.instr_type {
                InstrType::Extend => 0o0006,
                InstrType::Inhint => 0o0004,
                InstrType::Relint => 0o0003,
                InstrType::Go     => 0o4000,
                InstrType::Resume => {
                    // RESUME: opc5=0o05, qtr=0, addr7=0o017
                    // = (0o05 << 10) | (0 << 7) | 0o017
                    return Ok((0o05u16 << 10) | 0o017);
                }
                InstrType::Rupt   => {
                    // RUPT: opc3=0o10 => word = 0o10 << 12
                    return Ok(0o10u16 << 12);
                }
                _ => {
                    return Err(format!(
                        "special instruction '{}' has no known fixed address",
                        spec.mnemonic
                    ));
                }
            };
            // TC-form: opc3=0, addr12=fixed_addr
            fixed_addr & 0x7FFF
        }
        OpcodeFormat::Hardware => {
            return Err(format!(
                "instruction '{}' is hardware-triggered and cannot be assembled",
                spec.mnemonic
            ));
        }
    };
    Ok(word & 0x7FFF)
}

/// Parse an address token: accepts decimal or `0oNNN` octal.
fn parse_addr(token: &str) -> Result<u16, String> {
    if let Some(oct) = token.strip_prefix("0o") {
        u16::from_str_radix(oct, 8)
            .map_err(|e| format!("invalid octal address '{}': {}", token, e))
    } else {
        token
            .parse::<u16>()
            .map_err(|e| format!("invalid address '{}': {}", token, e))
    }
}

/// Assemble a text program into a list of 15-bit machine words.
///
/// Text format (one instruction per line):
/// ```text
/// # comment
/// CA  0o200        # octal address
/// AD  512          # decimal address
/// CCS 0o50         # extracode — EXTEND word auto-prepended
/// EXTEND           # may also be written explicitly
/// TC  0o4500
/// ```
///
/// Rules:
/// - Lines starting with `#` are comments (after stripping leading whitespace)
/// - Mnemonic is first token (case-insensitive)
/// - Address is second token: decimal or `0oNNN` octal
/// - Instructions that require EXTEND automatically get an EXTEND word prepended,
///   unless the previous effective word was already an EXTEND
/// - Explicit EXTEND lines are kept as-is (no double-EXTEND)
/// - Fixed-address instructions ignore any user-supplied address
///
/// Returns `Ok(Vec<u16>)` on success, `Err(String)` on the first error encountered.
pub fn assemble(text: &str) -> Result<Vec<u16>, String> {
    let specs = builtin_spec_set();
    let mut words: Vec<u16> = Vec::new();
    // Track whether the last emitted word was an EXTEND, so we don't double-emit.
    let mut last_was_extend = false;

    for (lineno, raw_line) in text.lines().enumerate() {
        // Strip inline comments.
        let line = match raw_line.find('#') {
            Some(i) => &raw_line[..i],
            None => raw_line,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut tokens = line.split_whitespace();
        let mnemonic = tokens.next().unwrap(); // guaranteed non-empty after trim

        let spec = specs
            .by_mnemonic(mnemonic)
            .ok_or_else(|| format!("line {}: unknown mnemonic '{}'", lineno + 1, mnemonic))?;

        // Parse address (optional for no-operand / special instructions).
        let addr: u16 = match tokens.next() {
            Some(tok) => parse_addr(tok)?,
            None => 0,
        };

        // Auto-prepend EXTEND if required and not already pending.
        if spec.requires_extend && !last_was_extend {
            words.push(EXTEND_WORD);
        }

        let word = encode_word(spec, addr)?;
        words.push(word);

        // Update EXTEND tracking: only the bare EXTEND instruction sets the flag.
        last_was_extend = spec.instr_type == InstrType::Extend;
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_extend_word() {
        let words = assemble("EXTEND").unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], EXTEND_WORD);
    }

    #[test]
    fn no_double_extend_on_explicit() {
        // Explicit EXTEND followed by CCS should produce exactly two words.
        let words = assemble("EXTEND\nCCS 0o50").unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], EXTEND_WORD);
    }

    #[test]
    fn auto_extend_ccs() {
        let words = assemble("CCS 0o50").unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], EXTEND_WORD);
    }

    #[test]
    fn encode_ca() {
        let words = assemble("CA 0o200").unwrap();
        assert_eq!(words.len(), 1);
        // CA: opc3=0o03, addr12=0o200
        // word = (3 << 12) | 0o200 = 0x3000 | 0x80 = 0x3080
        assert_eq!(words[0], (3u16 << 12) | 0o200);
    }

    #[test]
    fn encode_tc_fixed_addresses() {
        // EXTEND → TC 0o0006
        let e = assemble("EXTEND").unwrap();
        assert_eq!(e[0], 0o000006);

        // INHINT → TC 0o0004
        let i = assemble("INHINT").unwrap();
        assert_eq!(i[0], 0o000004);

        // RELINT → TC 0o0003
        let r = assemble("RELINT").unwrap();
        assert_eq!(r[0], 0o000003);

        // GO → TC 0o4000
        let g = assemble("GO").unwrap();
        assert_eq!(g[0], 0o004000);
    }

    /// NDX auto-prepends EXTEND (requires_extend=true).
    #[test]
    fn ndx_auto_extends() {
        let words = assemble("NDX 0o010").unwrap();
        assert_eq!(words.len(), 2, "NDX must emit EXTEND + NDX word");
        assert_eq!(words[0], EXTEND_WORD);
    }

    /// NDX with address ≤ 0o177 uses opcode 0o05 (7-bit erasable form).
    #[test]
    fn ndx_7bit_uses_opcode05() {
        use agc_interp::decode::decode;
        use agc_isa::builtin_spec_set;
        use agc_isa::InstrType;
        let words = assemble("NDX 0o077").unwrap();
        assert_eq!(words.len(), 2);
        let ndx_word = words[1];
        let s = builtin_spec_set();
        let d = decode(ndx_word, true, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Ndx);
        assert_eq!(d.address, 0o077);
    }

    /// NDX with address > 0o177 uses opcode 0o15 (10-bit wide form).
    #[test]
    fn ndx_10bit_uses_opcode15() {
        use agc_interp::decode::decode;
        use agc_isa::builtin_spec_set;
        use agc_isa::InstrType;
        let words = assemble("NDX 0o355").unwrap();
        assert_eq!(words.len(), 2);
        let ndx_word = words[1];
        let s = builtin_spec_set();
        let d = decode(ndx_word, true, &s).unwrap();
        assert_eq!(d.spec.instr_type, InstrType::Ndx);
        assert_eq!(d.address, 0o355, "10-bit round-trip failed");
    }
}
