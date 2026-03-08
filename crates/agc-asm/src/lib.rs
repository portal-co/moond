//! # agc-asm — AGC Block-2 assembler and disassembler
//!
//! Provides text-format assembly and disassembly of AGC Block-2 machine words.
//!
//! ## Assembler
//!
//! ```rust
//! use agc_asm::asm::assemble;
//!
//! let words = assemble("CA 0o200").unwrap();
//! assert_eq!(words.len(), 1);
//! ```
//!
//! ## Disassembler
//!
//! ```rust
//! use agc_asm::disasm::disassemble;
//!
//! let lines = disassemble(&[0o030200]);  // CA 0o200
//! assert!(lines[0].starts_with("CA"));
//! ```

pub mod asm;
pub mod disasm;

pub use asm::assemble;
pub use disasm::{disassemble, disassemble_word};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_ca() {
        let words = assemble("CA 0o200").unwrap();
        // CA is opcode 3, whole-3, no extend needed
        // word = 3 << 12 | 0o200 = 0x3000 | 0x80 = 0x3080
        assert_eq!(words.len(), 1);
        let lines = disassemble(&words);
        assert!(lines[0].starts_with("CA"), "got: {}", lines[0]);
    }

    #[test]
    fn autoextend_ccs() {
        let words = assemble("CCS 0o50").unwrap();
        assert_eq!(words.len(), 2); // EXTEND word + CCS word
        assert_eq!(words[0], 6);    // EXTEND = 0o000006
    }

    #[test]
    fn roundtrip_extend_then_ccs() {
        let words = assemble("CCS 0o50").unwrap();
        let text = disassemble(&words);
        // should get EXTEND + CCS lines
        assert_eq!(text.len(), 2);
        assert!(text[0].contains("EXTEND") || text[1].starts_with("CCS"),
                "got: {:?}", text);
    }

    #[test]
    fn explicit_extend_no_double() {
        // Explicit EXTEND followed by CCS should be 2 words, not 3.
        let words = assemble("EXTEND\nCCS 0o50").unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], 6);
    }

    #[test]
    fn roundtrip_tc() {
        let words = assemble("TC 0o100").unwrap();
        assert_eq!(words.len(), 1);
        let lines = disassemble(&words);
        assert!(lines[0].starts_with("TC"), "got: {}", lines[0]);
    }

    #[test]
    fn roundtrip_ad() {
        let words = assemble("AD 0o150").unwrap();
        assert_eq!(words.len(), 1);
        let lines = disassemble(&words);
        assert!(lines[0].starts_with("AD"), "got: {}", lines[0]);
    }

    #[test]
    fn roundtrip_dca() {
        // DCA is an extracode (requires_extend=true), Quarter5 format.
        let words = assemble("DCA 0o100").unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], 6); // EXTEND
        let lines = disassemble(&words);
        assert_eq!(lines[0], "EXTEND");
        assert!(lines[1].starts_with("DCA"), "got: {}", lines[1]);
    }

    #[test]
    fn special_instructions_no_address() {
        // INHINT, RELINT, GO, EXTEND are special fixed-address instructions.
        for mnem in &["INHINT", "RELINT", "GO", "EXTEND"] {
            let words = assemble(mnem).unwrap();
            // These do not require EXTEND themselves.
            assert_eq!(words.len(), 1, "{} produced wrong word count", mnem);
        }
    }

    #[test]
    fn comment_and_blank_lines_ignored() {
        let prog = "
# This is a comment
CA 0o200
# another comment
AD 0o150
";
        let words = assemble(prog).unwrap();
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn decimal_address() {
        // AD 128 decimal == AD 0o200
        let words_dec = assemble("AD 128").unwrap();
        let words_oct = assemble("AD 0o200").unwrap();
        assert_eq!(words_dec, words_oct);
    }

    #[test]
    fn roundtrip_read_channel() {
        // READ is a Channel6 instruction, requires EXTEND.
        let words = assemble("READ 0o30").unwrap();
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], 6); // EXTEND
        let lines = disassemble(&words);
        assert_eq!(lines[0], "EXTEND");
        assert!(lines[1].starts_with("READ"), "got: {}", lines[1]);
    }
}
