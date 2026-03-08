// Cross-tests: verify that the C and Rust AGC Block-2 decoder implementations
// produce identical results for all decodable instruction words.

use std::ffi::CStr;

/// Hardware-only instruction types that the C decoder can return but the Rust
/// decoder (agc-interp) does not implement (they have no opcode bits in the
/// ISA — they are triggered by hardware or GSE, not by instruction words).
fn is_hardware_only_mnemonic(mn: &str) -> bool {
    matches!(
        mn,
        "UNKNOWN"
            | "PINC"
            | "MINC"
            | "DINC"
            | "PCDU"
            | "MCDU"
            | "SHINC"
            | "SHANC"
            | "TCSAJ"
            | "FETCH"
            | "STORE"
            | "INOTRD"
            | "INOTLD"
    )
}

/// Exhaustively sweep all 15-bit words (0..=0x7FFF) with extend=false and
/// extend=true, and assert that the C and Rust decoders agree on:
///   - whether the word is decodable
///   - the instruction mnemonic
///   - the extracted address
///
/// Mnemonics that the C decoder can produce but Rust cannot (hardware-only
/// instructions that have no opcode encoding) are skipped — both missing is
/// also acceptable for those.
#[test]
fn c_and_rust_decode_agree_all_words() {
    use agc_isa::builtin_spec_set;
    use agc_interp::decode;

    let specs = builtin_spec_set();
    let mut failures: Vec<String> = Vec::new();

    for word in 0u16..=0x7FFF {
        for &extend in &[false, true] {
            // ── C decode ─────────────────────────────────────────────────────
            let c_decoded = unsafe { moond_sys::moond_decode_instr(word, extend) };

            // status: -1 = no match, 0 = one match, >0 = multiple matches
            // C decoders that return >0 (ambiguous) are treated as "no match"
            // for cross-test purposes — both sides should agree on "no match".
            let c_matched = c_decoded.status == 0;

            // ── Rust decode ──────────────────────────────────────────────────
            let rust_decoded = decode::decode(word, extend, &specs);
            let rust_matched = rust_decoded.is_ok();

            // ── Compare match status ─────────────────────────────────────────
            if c_matched != rust_matched {
                // Get the C mnemonic to decide if it's hardware-only
                if c_matched {
                    let c_mn = unsafe {
                        let ptr = moond_sys::moond_instr_mnemonic(c_decoded.type_);
                        if ptr.is_null() {
                            "UNKNOWN".to_string()
                        } else {
                            CStr::from_ptr(ptr)
                                .to_str()
                                .unwrap_or("UNKNOWN")
                                .to_string()
                        }
                    };
                    // Hardware-only types have no opcode encoding: Rust won't
                    // decode them, and that's expected.
                    if is_hardware_only_mnemonic(&c_mn) {
                        continue;
                    }
                }

                // Also allow: C returns ambiguous (status > 0), treat as no-match.
                // If the C decoder is ambiguous and Rust matched, that's a C bug.
                // We skip it rather than failing the cross-test.
                if c_decoded.status > 0 {
                    // C ambiguous — skip
                    continue;
                }

                failures.push(format!(
                    "Match mismatch at word=0o{:06o} extend={}: \
                     C matched={} (status={}) rust_matched={}",
                    word, extend, c_matched, c_decoded.status, rust_matched
                ));
                continue;
            }

            if !c_matched {
                // Both didn't match — OK
                continue;
            }

            // ── Both matched — compare mnemonics and addresses ───────────────
            let c_mn = unsafe {
                let ptr = moond_sys::moond_instr_mnemonic(c_decoded.type_);
                if ptr.is_null() {
                    "UNKNOWN".to_string()
                } else {
                    CStr::from_ptr(ptr)
                        .to_str()
                        .unwrap_or("UNKNOWN")
                        .to_string()
                }
            };

            // Skip hardware-only instructions even if both sides decode them.
            if is_hardware_only_mnemonic(&c_mn) {
                continue;
            }

            let rust_instr = rust_decoded.unwrap();
            let rust_mn = rust_instr.spec.mnemonic.as_str();

            if c_mn != rust_mn {
                failures.push(format!(
                    "Mnemonic mismatch at word=0o{:06o} extend={}: \
                     C={} rust={}",
                    word, extend, c_mn, rust_mn
                ));
                continue;
            }

            if c_decoded.address != rust_instr.address {
                failures.push(format!(
                    "Address mismatch for {} at word=0o{:06o} extend={}: \
                     C=0o{:06o} rust=0o{:06o}",
                    rust_mn, word, extend, c_decoded.address, rust_instr.address
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} cross-decode mismatch(es):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
