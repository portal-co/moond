//! Disassembler for the AGC Block-2 ISA.

use agc_isa::{builtin_spec_set, AddrMode, InstrType, InstrSpecSet};
use agc_interp::decode::decode;

/// Disassemble a slice of 15-bit words into assembly text lines.
///
/// Each word produces one output line. EXTEND state is tracked automatically:
/// after an EXTEND word, the following word is decoded with `extend = true`.
pub fn disassemble(words: &[u16]) -> Vec<String> {
    let specs = builtin_spec_set();
    let mut lines = Vec::with_capacity(words.len());
    let mut extend = false;

    for &word in words {
        let (line, new_extend) = disassemble_word(word, extend, &specs);
        lines.push(line);
        extend = new_extend;
    }

    lines
}

/// Disassemble a single 15-bit word given the current EXTEND state.
///
/// Returns `(line, new_extend_state)`.
///
/// - If the decoded instruction is EXTEND, the returned `new_extend_state` is `true`.
/// - Otherwise `new_extend_state` is `false` (EXTEND is consumed by this word).
pub fn disassemble_word(word: u16, extend: bool, specs: &InstrSpecSet) -> (String, bool) {
    let word = word & 0x7FFF;

    match decode(word, extend, specs) {
        Ok(decoded) => {
            let mnemonic = decoded.spec.mnemonic.as_str();
            let is_extend = decoded.spec.instr_type == InstrType::Extend;

            // Format the line.
            let line = if decoded.spec.addr_mode == AddrMode::None {
                // No operand instructions.
                mnemonic.to_string()
            } else {
                format!("{} 0o{:04o}", mnemonic, decoded.address)
            };

            // Advance extend state: EXTEND sets it, anything else clears it.
            let new_extend = is_extend;
            (line, new_extend)
        }
        Err(e) => {
            // Unknown encoding — emit a raw word comment.
            (format!("# UNKNOWN 0o{:05o} ({})", word, e.message), false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::assemble;

    #[test]
    fn disasm_extend() {
        let lines = disassemble(&[0o000006]);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "EXTEND");
    }

    #[test]
    fn disasm_ca() {
        // CA: opc3=0o03, addr=0o200
        let word = (3u16 << 12) | 0o200;
        let lines = disassemble(&[word]);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("CA"), "got: {}", lines[0]);
    }

    #[test]
    fn disasm_extend_then_ccs() {
        let words = assemble("CCS 0o50").unwrap();
        assert_eq!(words.len(), 2);
        let lines = disassemble(&words);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "EXTEND");
        assert!(lines[1].starts_with("CCS"), "got: {}", lines[1]);
    }
}
