// Cross-test: verify that the C encoder (moond_encode_simple) and the Rust
// assembler (agc_asm::assemble) produce identical 15-bit wire words for a
// representative set of standard AGC Block-2 instructions.
//
// # Address encoding conventions
//
// For non-channel instructions the `addr` field is used directly by both
// the C and Rust encoders.
//
// For channel instructions (READ, WRITE, RAND, WAND, ROR, WOR, RXOR) there is
// an important difference between the two encoders:
//
//   - The **C** encoder (`moond_encode_simple`) takes the full 9-bit channel
//     address field, where the top 3 bits select the channel variant (0=READ,
//     1=WRITE, …) and the bottom 6 bits are the physical channel number.
//
//   - The **Rust** assembler (`agc_asm::assemble`) takes only the 6-bit
//     physical channel number; the variant is encoded implicitly from the
//     instruction spec's `quarter` field.
//
// To keep the test simple while making both sides agree we provide separate
// `c_addr` and `rust_addr` values for channel instructions.

use std::ffi::CStr;

/// Map a mnemonic string to the corresponding C `moond_instr_type` constant.
fn c_instr_type(mnemonic: &str) -> Option<moond_sys::moond_instr_type> {
    use moond_sys::*;
    match mnemonic {
        "TC"     => Some(moond_instr_type_INSTR_TC),
        "TCF"    => Some(moond_instr_type_INSTR_TCF),
        "CCS"    => Some(moond_instr_type_INSTR_CCS),
        "BZF"    => Some(moond_instr_type_INSTR_BZF),
        "BZMF"   => Some(moond_instr_type_INSTR_BZMF),
        "CA"     => Some(moond_instr_type_INSTR_CA),
        "CS"     => Some(moond_instr_type_INSTR_CS),
        "DCA"    => Some(moond_instr_type_INSTR_DCA),
        "DCS"    => Some(moond_instr_type_INSTR_DCS),
        "TS"     => Some(moond_instr_type_INSTR_TS),
        "XCH"    => Some(moond_instr_type_INSTR_XCH),
        "LXCH"   => Some(moond_instr_type_INSTR_LXCH),
        "QXCH"   => Some(moond_instr_type_INSTR_QXCH),
        "DXCH"   => Some(moond_instr_type_INSTR_DXCH),
        "NDX"    => Some(moond_instr_type_INSTR_NDX),
        "AD"     => Some(moond_instr_type_INSTR_AD),
        "SU"     => Some(moond_instr_type_INSTR_SU),
        "MP"     => Some(moond_instr_type_INSTR_MP),
        "DV"     => Some(moond_instr_type_INSTR_DV),
        "ADS"    => Some(moond_instr_type_INSTR_ADS),
        "DAS"    => Some(moond_instr_type_INSTR_DAS),
        "INCR"   => Some(moond_instr_type_INSTR_INCR),
        "AUG"    => Some(moond_instr_type_INSTR_AUG),
        "DIM"    => Some(moond_instr_type_INSTR_DIM),
        "MSU"    => Some(moond_instr_type_INSTR_MSU),
        "MSK"    => Some(moond_instr_type_INSTR_MSK),
        "READ"   => Some(moond_instr_type_INSTR_READ),
        "WRITE"  => Some(moond_instr_type_INSTR_WRITE),
        "RAND"   => Some(moond_instr_type_INSTR_RAND),
        "WAND"   => Some(moond_instr_type_INSTR_WAND),
        "ROR"    => Some(moond_instr_type_INSTR_ROR),
        "WOR"    => Some(moond_instr_type_INSTR_WOR),
        "RXOR"   => Some(moond_instr_type_INSTR_RXOR),
        "EXTEND" => Some(moond_instr_type_INSTR_EXTEND),
        "INHINT" => Some(moond_instr_type_INSTR_INHINT),
        "RELINT" => Some(moond_instr_type_INSTR_RELINT),
        "RESUME" => Some(moond_instr_type_INSTR_RESUME),
        "GO"     => Some(moond_instr_type_INSTR_GO),
        _        => None,
    }
}

/// Encode one instruction word using the C `moond_encode_simple` function.
///
/// `c_addr` is passed directly to the C encoder, which for channel
/// instructions includes the variant in the upper 3 bits of the 9-bit field.
fn c_encode(mnemonic: &str, c_addr: u16) -> Result<u16, String> {
    let ty = c_instr_type(mnemonic)
        .ok_or_else(|| format!("no C type for mnemonic '{}'", mnemonic))?;

    let result = unsafe { moond_sys::moond_encode_simple(ty, c_addr) };

    if result.success {
        Ok(result.word)
    } else {
        let err = if result.error.is_null() {
            "unknown error".to_string()
        } else {
            unsafe { CStr::from_ptr(result.error).to_str().unwrap_or("?").to_string() }
        };
        Err(format!("C encode failed for {} 0o{:o}: {}", mnemonic, c_addr, err))
    }
}

/// Encode one instruction word using the Rust assembler (`agc_asm::assemble`).
///
/// Returns the last word emitted, which is always the instruction word
/// (EXTEND is auto-prepended for extracodes but we only want the instr word).
///
/// `rust_addr` is the operand passed to the assembler.  For channel
/// instructions this is the physical channel number only (no variant bits).
fn rust_encode(mnemonic: &str, rust_addr: u16) -> Result<u16, String> {
    let text = if rust_addr == 0 {
        mnemonic.to_string()
    } else {
        format!("{} 0o{:o}", mnemonic, rust_addr)
    };

    let words = agc_asm::assemble(&text)
        .map_err(|e| format!("Rust assemble failed for '{}': {}", text, e))?;

    words
        .last()
        .copied()
        .ok_or_else(|| format!("Rust assemble produced no words for '{}'", text))
}

/// Test that the C encoder and Rust assembler produce the same 15-bit wire word
/// for a representative set of AGC Block-2 instructions.
///
/// For channel instructions, separate `c_addr` / `rust_addr` values are
/// provided because the two encoders use different address conventions (see
/// module-level doc comment).
#[test]
fn c_and_rust_encode_agree_for_standard_instrs() {
    // (mnemonic, c_addr, rust_addr)
    // For non-channel instructions c_addr == rust_addr.
    // For channel instructions:
    //   c_addr   = (variant << 6) | channel   (full 9-bit field for C encoder)
    //   rust_addr = channel                    (Rust encoder adds variant from spec)
    let cases: &[(&str, u16, u16)] = &[
        // ── Whole-3 (3-bit opcode + 12-bit address) ──────────────────────────
        ("TC",     0o4500, 0o4500),
        ("CA",     0o200,  0o200),
        ("CS",     0o100,  0o100),
        ("AD",     0o300,  0o300),
        ("MSK",    0o150,  0o150),
        // ── Quarter-5, no EXTEND ──────────────────────────────────────────────
        ("TCF",    0o100,  0o100),
        // ── Quarter-5 extracodes (address is 7-bit: max 0o177) ────────────────
        ("CCS",    0o050,  0o050),
        ("XCH",    0o100,  0o100),
        // ADS: max addr is 0o177 (7-bit); use 0o020 instead of 0o200
        ("ADS",    0o020,  0o020),
        ("MP",     0o150,  0o150),
        ("DV",     0o030,  0o030),
        ("DAS",    0o010,  0o010),
        ("LXCH",   0o020,  0o020),
        ("INCR",   0o030,  0o030),
        ("AUG",    0o040,  0o040),
        ("DIM",    0o050,  0o050),
        ("MSU",    0o060,  0o060),
        ("QXCH",   0o070,  0o070),
        ("DXCH",   0o100,  0o100),
        ("TS",     0o110,  0o110),
        ("SU",     0o020,  0o020),
        ("BZF",    0o010,  0o010),
        ("BZMF",   0o010,  0o010),
        ("DCA",    0o100,  0o100),
        ("DCS",    0o100,  0o100),
        ("NDX",    0o100,  0o100),
        // ── Channel-6 (requires EXTEND) ───────────────────────────────────────
        // Physical channel number = 0o030.
        // C encoder takes a 9-bit field: bits[8:6] = variant, bits[5:0] = channel.
        // Rust encoder takes just the channel number; variant from spec.quarter.
        // variant 0 = READ, 1 = WRITE, 2 = RAND, 3 = WAND, 4 = ROR, 5 = WOR, 6 = RXOR
        ("READ",   0o000 | 0o030,  0o030),   // c_addr = (0<<6)|0o30, rust = 0o30
        ("WRITE",  0o100 | 0o030,  0o030),   // c_addr = (1<<6)|0o30, rust = 0o30
        ("RAND",   0o200 | 0o030,  0o030),   // c_addr = (2<<6)|0o30, rust = 0o30
        ("WAND",   0o300 | 0o030,  0o030),   // c_addr = (3<<6)|0o30, rust = 0o30
        ("ROR",    0o400 | 0o030,  0o030),   // c_addr = (4<<6)|0o30, rust = 0o30
        ("WOR",    0o500 | 0o030,  0o030),   // c_addr = (5<<6)|0o30, rust = 0o30
        ("RXOR",   0o600 | 0o030,  0o030),   // c_addr = (6<<6)|0o30, rust = 0o30
        // ── Special fixed-address ─────────────────────────────────────────────
        ("EXTEND", 0, 0),
        ("INHINT", 0, 0),
        ("RELINT", 0, 0),
        ("GO",     0, 0),
    ];

    let mut failures: Vec<String> = Vec::new();

    for &(mnemonic, c_addr, rust_addr) in cases {
        let c_word = match c_encode(mnemonic, c_addr) {
            Ok(w)  => w,
            Err(e) => { failures.push(e); continue; }
        };

        let rust_word = match rust_encode(mnemonic, rust_addr) {
            Ok(w)  => w,
            Err(e) => { failures.push(e); continue; }
        };

        if c_word != rust_word {
            failures.push(format!(
                "Encode mismatch for {} (c_addr=0o{:o} rust_addr=0o{:o}): \
                 C=0o{:06o} (0x{:04x}) Rust=0o{:06o} (0x{:04x})",
                mnemonic, c_addr, rust_addr,
                c_word, c_word, rust_word, rust_word
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} encode cross-test failure(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
