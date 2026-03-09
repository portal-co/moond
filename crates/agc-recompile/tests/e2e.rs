//! End-to-end tests for the AGC recompiler (C and WASM backends).
//!
//! Each test:
//! 1. Constructs a small AGC memory image (by encoding words directly or via
//!    `agc_asm::assemble`).
//! 2. For C tests: runs the frontend (`decode_stream`) to obtain an `InstrStream` IR,
//!    then drives the C backend and verifies the output.
//! 3. For WASM tests: feeds all 4096 × 2 instructions into `WasmDirectBackend`
//!    and validates the output with `wasmparser::Validator`.

use std::collections::BTreeSet;

use agc_recompile::{
    backend::{
        c::CBackend,
        wasm::WasmDirectBackend,
        Backend, DirectBackend,
    },
    decode_direct, decode_stream, InstrStream, Terminator,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build a blank 4096-word AGC memory image (all zeros ≡ TC 0 = jump to 0).
fn blank_image() -> Box<[u16; 4096]> {
    Box::new([0u16; 4096])
}

/// Write `words` into `image` starting at AGC (12-bit) address `start`.
fn place(image: &mut [u16; 4096], start: u16, words: &[u16]) {
    for (i, &w) in words.iter().enumerate() {
        image[(start as usize + i) & 0x0FFF] = w;
    }
}

/// Assemble one line of AGC asm and return the raw word(s).
fn asm(line: &str) -> Vec<u16> {
    agc_asm::assemble(line).unwrap_or_else(|e| panic!("assemble({line:?}): {e}"))
}

/// Validate `bytes` as a well-formed WebAssembly module.
fn validate_wasm(bytes: &[u8]) {
    use wasmparser::{Validator, WasmFeatures};
    let features = WasmFeatures::all();
    let mut v = Validator::new_with_features(features);
    v.validate_all(bytes)
        .unwrap_or_else(|e| panic!("WASM validation failed: {e}"));
}

/// Run `decode_stream` on `image` with `entry` and no indirect targets.
fn decode(image: &[u16; 4096], entry: u16) -> InstrStream {
    decode_stream(image, &[entry], BTreeSet::new())
        .unwrap_or_else(|e| panic!("frontend error: {e}"))
}

/// Feed all 4096 × 2 instructions into `WasmDirectBackend` and return the
/// assembled WASM module bytes.
fn make_wasm(image: &[u16; 4096], entry_points: &[u16]) -> Vec<u8> {
    let mut backend = WasmDirectBackend::new(entry_points.to_vec());
    for addr in 0u16..4096 {
        for &extend in &[false, true] {
            let instr = decode_direct(image, addr, extend, &BTreeSet::new());
            backend.feed_instr(&instr).unwrap();
        }
    }
    backend.finish().unwrap()
}

// ─── C backend tests ──────────────────────────────────────────────────────────

/// A TC instruction at 0o4000 jumping to 0o4002 should produce exactly two
/// basic blocks (0o4000 and 0o4002 — 0o4001 is dead), with `goto bb_04002` in
/// the first block.
#[test]
fn c_tcf_jump_produces_two_blocks() {
    let mut img = blank_image();
    // TC (12-bit address) to 0o4002; TCF only has a 7-bit address field which
    // would truncate 0o4002 to address 2.
    let tc_to_4002 = asm("TC 0o4002");
    let tc_loop    = asm("TC 0o4002");
    place(&mut img, 0o4000, &tc_to_4002);
    // 0o4001 is intentionally left as zero (dead)
    place(&mut img, 0o4002, &tc_loop);

    let stream = decode(&img, 0o4000);

    // Exactly 2 blocks: 0o4000 and 0o4002.
    assert_eq!(stream.blocks.len(), 2, "expected 2 blocks, got: {:?}", stream.blocks.keys().collect::<Vec<_>>());
    assert!(stream.blocks.contains_key(&0o4000));
    assert!(stream.blocks.contains_key(&0o4002));

    // 0o4000's terminator is Jump(0o4002).
    assert!(
        matches!(stream.blocks[&0o4000].terminator, Terminator::Jump(0o4002)),
        "expected Jump(0o4002), got {:?}", stream.blocks[&0o4000].terminator
    );

    let c = CBackend::default().emit(&stream).unwrap();

    // Must contain both block labels.
    assert!(c.contains("bb_04000:"), "missing bb_04000 label");
    assert!(c.contains("bb_04002:"), "missing bb_04002 label");
    // Must NOT contain the dead block label.
    assert!(!c.contains("bb_04001"), "dead block 0o4001 should not appear");
    // The first block must jump to the second.
    assert!(c.contains("goto bb_04002"), "missing jump to bb_04002");
    // Runtime header must be present.
    assert!(c.contains("AgcState"), "missing AgcState in output");
}

/// A CA instruction followed by TC produces two adjacent basic blocks: one for
/// CA (FallThrough to next) and one for TC (Jump back).  The frontend creates a
/// new block at every branch/fall-through target, so sequential instructions
/// each get their own block.
///
/// Placed at erasable 0o0100 to avoid TC 0o4000 → Go special case.
#[test]
fn c_ca_then_tcf_block_structure() {
    let mut img = blank_image();
    place(&mut img, 0o0100, &asm("CA 0o010"));   // load erasable[0o010] into A
    place(&mut img, 0o0101, &asm("TC 0o0100"));  // loop back to 0o0100

    let stream = decode(&img, 0o0100);

    // Two blocks: CA block at 0o0100 (FallThrough to 0o0101), TC block at 0o0101 (Jump back).
    assert_eq!(stream.blocks.len(), 2);
    assert!(stream.blocks.contains_key(&0o0100));
    assert!(stream.blocks.contains_key(&0o0101));

    // CA block has one instruction and falls through.
    let ca_block = &stream.blocks[&0o0100];
    assert_eq!(ca_block.instrs.len(), 1);
    assert_eq!(ca_block.instrs[0].mnemonic, "CA");
    assert!(matches!(ca_block.terminator, Terminator::FallThrough(0o0101)));

    // TC block has one instruction and jumps back.
    let tc_block = &stream.blocks[&0o0101];
    assert_eq!(tc_block.instrs.len(), 1);
    assert_eq!(tc_block.instrs[0].mnemonic, "TC");
    assert!(matches!(tc_block.terminator, Terminator::Jump(0o0100)));

    let c = CBackend::default().emit(&stream).unwrap();
    assert!(c.contains("bb_00100:"));
    assert!(c.contains("bb_00101:"));
    assert!(c.contains("goto bb_00100"));
}

/// CCS (Count, Compare and Skip) should produce a basic block whose terminator
/// is a CcsBranch with 4 distinct targets, and the C output should contain a
/// switch over s->z covering all four.
#[test]
fn c_ccs_emits_four_way_switch() {
    let mut img = blank_image();
    // CCS 0o100 — requires EXTEND prefix.
    let ccs = asm("CCS 0o100");
    place(&mut img, 0o4000, &ccs);
    // The 4 targets are 0o4002, 0o4003, 0o4004, 0o4005 (for positive,
    // plus-zero, negative, minus-zero results).  Fill them with self-loops.
    for t in [0o4002u16, 0o4003, 0o4004, 0o4005] {
        let tcf = asm(&format!("TCF 0o{t:o}"));
        place(&mut img, t, &tcf);
    }

    // Seed entry at 0o4000 where CCS lives.  (CCS is 2 words: EXTEND + CCS.)
    let stream = decode(&img, 0o4000);

    // 0o4000 block should have a CcsBranch terminator.
    let bb = &stream.blocks[&0o4000];
    assert!(
        matches!(bb.terminator, Terminator::CcsBranch(_)),
        "expected CcsBranch, got {:?}", bb.terminator
    );

    let c = CBackend::default().emit(&stream).unwrap();
    // The switch on s->z must appear.
    assert!(c.contains("switch (s->z)"), "missing switch on s->z");
    // All four target labels must be reachable from the dispatch.
    assert!(c.contains("bb_04002"), "missing target 0o4002");
    assert!(c.contains("bb_04003"), "missing target 0o4003");
    assert!(c.contains("bb_04004"), "missing target 0o4004");
    assert!(c.contains("bb_04005"), "missing target 0o4005");
}

/// An empty stream (no blocks) produces valid compilable C with the expected
/// function signature and immediate AGC_HALT return.
#[test]
fn c_empty_stream_compiles() {
    let stream = InstrStream {
        blocks: std::collections::BTreeMap::new(),
        entry_points: vec![],
        indirect_targets: BTreeSet::new(),
    };
    let c = CBackend::default().emit(&stream).unwrap();
    assert!(c.contains("int agc_run(AgcState *s)"));
    assert!(c.contains("AGC_HALT"));
    // No block labels in an empty stream.
    assert!(!c.contains("bb_"));
}

// ─── WASM backend tests ───────────────────────────────────────────────────────

/// The WASM backend must produce a valid, parseable WebAssembly module for a
/// simple program (single TCF self-loop).
#[test]
fn wasm_tcf_loop_is_valid_wasm() {
    let mut img = blank_image();
    let tcf = asm("TCF 0o4000");
    place(&mut img, 0o4000, &tcf);

    let bytes = make_wasm(&img, &[0o4000]);

    assert!(!bytes.is_empty(), "WASM output is empty");
    assert_eq!(&bytes[0..4], b"\0asm", "missing WASM magic");
    validate_wasm(&bytes);
}

/// A two-block program with a direct jump validates as well-formed WASM, and
/// the module exports a function for each entry point.
#[test]
fn wasm_two_block_program_is_valid_and_exports_entry() {
    let mut img = blank_image();
    let tcf_4002 = asm("TCF 0o4002");
    let tcf_loop  = asm("TCF 0o4002");
    place(&mut img, 0o4000, &tcf_4002);
    place(&mut img, 0o4002, &tcf_loop);

    let bytes = make_wasm(&img, &[0o4000]);

    validate_wasm(&bytes);

    // The entry point (0o4000) should be exported as "bb_04000".
    assert!(
        bytes.windows(8).any(|w| w == b"bb_04000"),
        "export 'bb_04000' not found in WASM binary"
    );
}

/// A CA + TCF program produces valid WASM.
#[test]
fn wasm_ca_then_tcf_is_valid_wasm() {
    let mut img = blank_image();
    place(&mut img, 0o4000, &asm("CA 0o100"));
    place(&mut img, 0o4001, &asm("TCF 0o4000"));

    let bytes = make_wasm(&img, &[0o4000]);
    validate_wasm(&bytes);
}

/// A CCS instruction (requiring EXTEND, producing a 4-way branch) produces
/// valid WASM with exactly 8192 generated functions (4096 addrs × 2 states).
#[test]
fn wasm_ccs_four_way_branch_is_valid_wasm() {
    let mut img = blank_image();
    let ccs = asm("CCS 0o100");
    place(&mut img, 0o4000, &ccs);
    for t in [0o4002u16, 0o4003, 0o4004, 0o4005] {
        place(&mut img, t, &asm(&format!("TCF 0o{t:o}")));
    }

    let bytes = make_wasm(&img, &[0o4000]);
    validate_wasm(&bytes);

    // The direct backend always emits 8192 functions (4096 × 2 extend states).
    use wasmparser::Parser;
    let mut code_count = 0usize;
    for payload in Parser::new(0).parse_all(&bytes) {
        if let wasmparser::Payload::CodeSectionEntry(_) = payload.unwrap() {
            code_count += 1;
        }
    }
    assert_eq!(code_count, 8192, "expected 8192 functions in code section");
}

// ─── Cross-backend consistency ────────────────────────────────────────────────

/// Both backends must accept exactly the same program without error, and
/// must produce non-empty outputs, for a multi-instruction program.
#[test]
fn both_backends_accept_multi_instruction_program() {
    // Three sequential base instructions at erasable addresses.
    // XCH is an extended instruction (requires EXTEND prefix), so we use AD
    // instead to avoid EXTEND-state complications.
    // Placed at 0o0100 to avoid TC 0o4000 → Go special case.
    let mut img = blank_image();
    place(&mut img, 0o0100, &asm("CA 0o010"));   // load
    place(&mut img, 0o0101, &asm("AD 0o010"));   // add mem[0o010] to A
    place(&mut img, 0o0102, &asm("TC 0o0100"));  // loop back

    // Each non-EXTEND instruction is its own block; the three-instruction
    // program decodes to 3 blocks (CA → FT → AD → FT → TC → Jump back).
    let stream = decode(&img, 0o0100);
    assert_eq!(stream.blocks.len(), 3, "expected 3 blocks (one per instruction)");

    let c    = CBackend::default().emit(&stream).unwrap();
    let wasm = make_wasm(&img, &[0o0100]);

    assert!(!c.is_empty(),    "C backend produced empty output");
    assert!(!wasm.is_empty(), "WASM backend produced empty output");
    validate_wasm(&wasm);
    // C output should reference all three mnemonics.
    assert!(c.contains("CA"), "C output missing CA mnemonic comment");
    assert!(c.contains("AD"), "C output missing AD mnemonic comment");
    assert!(c.contains("TC"), "C output missing TC mnemonic comment");
}
