// Integration tests for moond-sys FFI bindings.
// Ports of the C test programs: bits_test.c and decode_test.c

mod bits_tests {
    // The C bits.h functions (agc_to_c_bit, extract_agc_bits, etc.) are
    // static inline functions and therefore not exported to FFI bindings.
    // We re-implement them here in Rust, mirroring bits.h exactly, and
    // verify the same invariants the C tests check.

    // ── Bit-numbering conversion ──────────────────────────────────────────────

    /// AGC bit N (1=MSB, 15=LSB) → C bit number (14=MSB, 0=LSB).
    /// Formula: c_bit = 15 - agc_bit  (bits.h: agc_to_c_bit)
    fn agc_to_c_bit(agc_bit: u8) -> u8 {
        15 - agc_bit
    }

    /// C bit number → AGC bit number.
    /// Formula: agc_bit = 15 - c_bit  (bits.h: c_to_agc_bit)
    fn c_to_agc_bit(c_bit: u8) -> u8 {
        15 - c_bit
    }

    // ── Bit field extraction / insertion ─────────────────────────────────────

    /// Extract AGC bit range [start_bit..=end_bit] from a 15-bit word,
    /// right-justified.  NO bit reversal.  (bits.h: extract_agc_bits)
    fn extract_agc_bits(word: u16, start_bit: u8, end_bit: u8) -> u16 {
        let c_start = agc_to_c_bit(start_bit); // higher C bit
        let c_end   = agc_to_c_bit(end_bit);   // lower  C bit
        let width   = c_start - c_end + 1;
        let mask    = (1u16 << width) - 1;
        (word >> c_end) & mask
    }

    /// Insert `value` into AGC bit range [start_bit..=end_bit] of `word`.
    /// NO bit reversal.  (bits.h: insert_agc_bits)
    fn insert_agc_bits(word: u16, value: u16, start_bit: u8, end_bit: u8) -> u16 {
        let c_start = agc_to_c_bit(start_bit);
        let c_end   = agc_to_c_bit(end_bit);
        let width   = c_start - c_end + 1;
        let mask    = (1u16 << width) - 1;
        (word & !(mask << c_end)) | ((value & mask) << c_end)
    }

    /// Create a word from a value placed in the given AGC bit range, zeros elsewhere.
    /// (bits.h: make_agc_bits)
    fn make_agc_bits(value: u16, start_bit: u8, end_bit: u8) -> u16 {
        insert_agc_bits(0, value, start_bit, end_bit)
    }

    /// Reverse the low `num_bits` bits of `value`.  (bits.h: reverse_bits)
    fn reverse_bits(mut value: u16, num_bits: u8) -> u16 {
        let mut result = 0u16;
        for _ in 0..num_bits {
            result = (result << 1) | (value & 1);
            value >>= 1;
        }
        result
    }

    /// Extract AGC bit range WITH bit reversal (AGC-semantic).
    /// (bits.h: extract_agc_bits_reversed)
    fn extract_agc_bits_reversed(word: u16, start_bit: u8, end_bit: u8) -> u16 {
        let num_bits = end_bit - start_bit + 1;
        let value    = extract_agc_bits(word, start_bit, end_bit);
        reverse_bits(value, num_bits)
    }

    /// Insert AGC bit range WITH bit reversal (AGC-semantic).
    /// (bits.h: insert_agc_bits_reversed)
    fn insert_agc_bits_reversed(word: u16, value: u16, start_bit: u8, end_bit: u8) -> u16 {
        let num_bits       = end_bit - start_bit + 1;
        let reversed_value = reverse_bits(value, num_bits);
        insert_agc_bits(word, reversed_value, start_bit, end_bit)
    }

    // ── Test functions ────────────────────────────────────────────────────────

    /// Port of test_bit_conversion() from bits_test.c
    #[test]
    fn test_bit_conversion() {
        // AGC bit 1 = MSB = C bit 14
        assert_eq!(agc_to_c_bit(1), 14, "AGC bit 1 -> C bit 14");
        // AGC bit 15 = LSB = C bit 0
        assert_eq!(agc_to_c_bit(15), 0, "AGC bit 15 -> C bit 0");
        // AGC bit 7 = C bit 8
        assert_eq!(agc_to_c_bit(7), 8, "AGC bit 7 -> C bit 8");

        // Inverse: c_to_agc_bit is the same formula
        assert_eq!(c_to_agc_bit(14), 1, "C bit 14 -> AGC bit 1");
        assert_eq!(c_to_agc_bit(0), 15, "C bit 0 -> AGC bit 15");
        assert_eq!(c_to_agc_bit(8), 7, "C bit 8 -> AGC bit 7");
    }

    /// Port of test_extract_bits() from bits_test.c
    #[test]
    fn test_extract_bits() {
        // Test word: 030050 octal = CA K 00050
        // Bits 1-3 (opcode) should be 03 octal
        // Bits 4-15 (address) should be 00050 octal
        let word: u16 = 0o030050;

        let opcode = extract_agc_bits(word, 1, 3);
        assert_eq!(
            opcode, 0o03,
            "Extract bits 1-3 from 0o{:05o}: got 0o{:02o} (expected 0o03)",
            word, opcode
        );

        let addr = extract_agc_bits(word, 4, 15);
        assert_eq!(
            addr, 0o00050,
            "Extract bits 4-15 from 0o{:05o}: got 0o{:05o} (expected 0o00050)",
            word, addr
        );

        // Test word: 017234 octal = MP K 0234
        // Bits 1-6 (opcode) should be 17 octal
        // Bits 7-15 (address) should be 0234 octal
        let word: u16 = 0o017234;

        let opcode6 = extract_agc_bits(word, 1, 6);
        assert_eq!(
            opcode6, 0o17,
            "Extract bits 1-6 from 0o{:05o}: got 0o{:02o} (expected 0o17)",
            word, opcode6
        );

        let addr9 = extract_agc_bits(word, 7, 15);
        assert_eq!(
            addr9, 0o0234,
            "Extract bits 7-15 from 0o{:05o}: got 0o{:04o} (expected 0o0234)",
            word, addr9
        );

        // Test word: 010030 octal = READ H 030
        // Bits 1-6 (opcode) should be 10 octal
        // Bits 7-9 (quarter) should be 0 octal
        // Bits 10-15 (address) should be 030 octal
        let word: u16 = 0o010030;

        let opcode_read = extract_agc_bits(word, 1, 6);
        assert_eq!(
            opcode_read, 0o10,
            "Extract bits 1-6 from 0o{:05o}: got 0o{:02o} (expected 0o10)",
            word, opcode_read
        );

        let quarter = extract_agc_bits(word, 7, 9);
        assert_eq!(
            quarter, 0,
            "Extract bits 7-9 from 0o{:05o}: got 0o{:o} (expected 0)",
            word, quarter
        );

        let addr6 = extract_agc_bits(word, 10, 15);
        assert_eq!(
            addr6, 0o030,
            "Extract bits 10-15 from 0o{:05o}: got 0o{:03o} (expected 0o030)",
            word, addr6
        );
    }

    /// Port of test_insert_bits() from bits_test.c
    #[test]
    fn test_insert_bits() {
        // Build CA K 00050 = 030050 octal
        // Insert 03 into bits 1-3, 00050 into bits 4-15
        let mut word: u16 = 0;
        word = insert_agc_bits(word, 0o03, 1, 3);
        word = insert_agc_bits(word, 0o00050, 4, 15);
        assert_eq!(word, 0o030050, "Built word 0o{:05o} (expected 0o030050)", word);

        // Build READ H 030 = 010030 octal
        // Insert 10 into bits 1-6, 0 into bits 7-9, 030 into bits 10-15
        let mut word: u16 = 0;
        word = insert_agc_bits(word, 0o10, 1, 6);
        word = insert_agc_bits(word, 0, 7, 9);
        word = insert_agc_bits(word, 0o030, 10, 15);
        assert_eq!(word, 0o010030, "Built word 0o{:05o} (expected 0o010030)", word);
    }

    /// Port of test_make_bits() from bits_test.c
    #[test]
    fn test_make_bits() {
        // Make CA opcode in bits 1-3: value=03 octal
        let word = make_agc_bits(0o03, 1, 3);
        assert_eq!(
            word, 0o030000,
            "make_agc_bits(0o03, 1, 3) = 0o{:05o} (expected 0o030000)",
            word
        );

        // Make READ opcode in bits 1-6: value=010 octal
        let word = make_agc_bits(0o010, 1, 6);
        assert_eq!(
            word, 0o010000,
            "make_agc_bits(0o010, 1, 6) = 0o{:05o} (expected 0o010000)",
            word
        );
    }

    /// Port of test_bit_reversal() from bits_test.c
    #[test]
    fn test_bit_reversal() {
        // reverse_bits(0b000001, 6) should give 0b100000 (32 decimal)
        let rev = reverse_bits(0b000001, 6);
        assert_eq!(
            rev, 0b100000,
            "reverse_bits(0b000001, 6) = 0b{:06b} (expected 0b100000)",
            rev
        );

        // reverse_bits(0b101, 3) should give 0b101 (palindrome)
        let rev = reverse_bits(0b101, 3);
        assert_eq!(
            rev, 0b101,
            "reverse_bits(0b101, 3) = 0b{:03b} (expected 0b101)",
            rev
        );

        // reverse_bits(0b001, 3) should give 0b100
        let rev = reverse_bits(0b001, 3);
        assert_eq!(
            rev, 0b100,
            "reverse_bits(0b001, 3) = 0b{:03b} (expected 0b100)",
            rev
        );

        // reverse_bits(01 octal, 6) -> 040 octal = 32 decimal
        let rev = reverse_bits(0o01, 6);
        assert_eq!(
            rev, 0o040,
            "reverse_bits(0o01, 6) = 0o{:03o} (expected 0o040)",
            rev
        );
    }

    /// Port of test_agc_semantic_operations() from bits_test.c
    #[test]
    fn test_agc_semantic_operations() {
        // If AGC documentation says order code "01" octal is in bits 1-6:
        //   "01" octal = 1 decimal = 0b000001
        //   AGC bit 1 (MSB of field) should contain the MSB of value = 0
        //   AGC bit 6 (LSB of field) should contain the LSB of value = 1
        //   In C: bit 14 = 0, bit 9 = 1
        //   So C bits 14-9 should be: 0b100000 (reversed!)
        let word: u16 = 0b100000u16 << 9; // Put reversed value in C bits 14-9
        let extracted = extract_agc_bits_reversed(word, 1, 6);
        assert_eq!(
            extracted, 0o01,
            "extract_agc_bits_reversed gave 0o{:02o} (expected 0o01); \
             word=0o{:05o}, C bits 14-9 = 0b{:06b}",
            extracted, word, (word >> 9) & 0x3F
        );

        // Test AGC-semantic insertion:
        // Insert opcode "01" octal into AGC bits 1-6
        // Should reverse it to 0b100000 and place in C bits 14-9
        let word = insert_agc_bits_reversed(0, 0o01, 1, 6);
        let c_bits = (word >> 9) & 0x3F;
        assert_eq!(
            c_bits, 0b100000,
            "insert_agc_bits_reversed gave C bits = 0b{:06b} (expected 0b100000)",
            c_bits
        );

        // Verify round-trip: extract what we inserted
        let extracted = extract_agc_bits_reversed(word, 1, 6);
        assert_eq!(
            extracted, 0o01,
            "round-trip failed: inserted 0o01, extracted 0o{:02o}",
            extracted
        );
    }
}

mod decode_tests {
    use std::ffi::CStr;

    /// Port of the decode_test.c exhaustive round-trip for extend=false.
    ///
    /// For every 16-bit word:
    ///   1. Decode with extend=false
    ///   2. If no match (status == -1), skip
    ///   3. If multiple matches (status > 0), fail
    ///   4. Encode result
    ///   5. Assert encoded word has no bits that original doesn't have
    #[test]
    fn exhaustive_roundtrip_no_extend() {
        exhaustive_roundtrip(false);
    }

    /// Port of the decode_test.c exhaustive round-trip for extend=true.
    #[test]
    fn exhaustive_roundtrip_with_extend() {
        exhaustive_roundtrip(true);
    }

    fn exhaustive_roundtrip(extend: bool) {
        let mut failures: Vec<String> = Vec::new();

        for i in 0u32..=0xFFFF {
            let orig_word = i as u16;

            // Step 1: Decode
            let decoded = unsafe { moond_sys::moond_decode_instr(orig_word, extend) };

            // Step 2/3: Check decoder status
            // status is i8: -1 means no match (i.e. decoded.status + 1 == 0),
            // 0 means exactly one match, positive means multiple matches.
            if decoded.status == -1 {
                // No match — skip silently (same as C test)
                continue;
            }
            if decoded.status > 0 {
                failures.push(format!(
                    "Decoder status={} (expected 0) for word=0x{:04x}/0o{:06o} extend={}",
                    decoded.status, orig_word, orig_word, extend
                ));
                continue;
            }

            // Step 4: Encode
            let encoded = unsafe { moond_sys::moond_encode_instr(&decoded as *const _) };

            if !encoded.success {
                let mnemonic = unsafe {
                    let ptr = moond_sys::moond_instr_mnemonic(decoded.type_);
                    if ptr.is_null() {
                        "<null>".to_string()
                    } else {
                        CStr::from_ptr(ptr).to_str().unwrap_or("<invalid>").to_string()
                    }
                };
                failures.push(format!(
                    "Encode failed for {} at word=0x{:04x}/0o{:06o} extend={}",
                    mnemonic, orig_word, orig_word, extend
                ));
                continue;
            }

            // Step 5: Round-trip check
            // Encoding can clear unused bits, but must NOT set new bits.
            let original_bits = orig_word;
            let encoded_bits  = encoded.word;
            let extra_bits    = encoded_bits & !original_bits;

            if extra_bits != 0 {
                let mnemonic = unsafe {
                    let ptr = moond_sys::moond_instr_mnemonic(decoded.type_);
                    if ptr.is_null() {
                        "<null>".to_string()
                    } else {
                        CStr::from_ptr(ptr).to_str().unwrap_or("<invalid>").to_string()
                    }
                };
                failures.push(format!(
                    "Round-trip mismatch for {} at word=0x{:04x}/0o{:06o} extend={}: \
                     encoded=0x{:04x}/0o{:06o}, extra_bits=0x{:04x}",
                    mnemonic, orig_word, orig_word, extend,
                    encoded_bits, encoded_bits, extra_bits
                ));
            }
        }

        if !failures.is_empty() {
            panic!(
                "{} round-trip failure(s) with extend={}:\n{}",
                failures.len(),
                extend,
                failures.join("\n")
            );
        }
    }
}
