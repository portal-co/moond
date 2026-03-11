//! End-to-end tests for the AGC recompiler (C and WASM backends).
//!
//! Each test:
//! 1. Constructs a small AGC memory image (by encoding words directly or via
//!    `agc_asm::assemble`).
//! 2. For C tests: runs the frontend (`decode_stream`) to obtain an `InstrStream` IR,
//!    then drives the C backend and compiles the output with the system C compiler.
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

/// Pipe `src` through the system C compiler (`cc -fsyntax-only`).
/// Panics with the compiler's stderr if compilation fails.
fn compile_c(src: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("cc")
        .args([
            "-x", "c", "-fsyntax-only",
            "-Wall",
            "-Wno-unused-function",
            "-Wno-unused-variable",
            "-Wno-unused-label",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn C compiler (cc); ensure cc is on PATH");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(src.as_bytes())
        .expect("failed to write C source to compiler");

    let out = child.wait_with_output().expect("C compiler wait failed");
    if !out.status.success() {
        panic!(
            "C compilation failed:\n{}\n--- source ---\n{}",
            String::from_utf8_lossy(&out.stderr),
            src
        );
    }
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

/// A TC instruction at 0o4000 jumping to 0o4002 (0o4001 is dead) must decode
/// with a Jump(0o4002) terminator on the entry block, and the generated C must
/// compile cleanly.
#[test]
fn c_tcf_jump_produces_jump_terminator() {
    let mut img = blank_image();
    // TC (12-bit address) to 0o4002; TCF only has a 7-bit address field which
    // would truncate 0o4002 to address 2.
    place(&mut img, 0o4000, &asm("TC 0o4002"));
    // 0o4001 intentionally left as zero (dead)
    place(&mut img, 0o4002, &asm("TC 0o4002"));

    let stream = decode(&img, 0o4000);

    // Entry block must be present.
    assert!(stream.blocks.contains_key(&0o4000), "entry block 0o4000 missing");

    // Entry block's terminator must be a direct jump to 0o4002.
    assert!(
        matches!(stream.blocks[&0o4000].terminator, Terminator::Jump(0o4002)),
        "expected Jump(0o4002), got {:?}", stream.blocks[&0o4000].terminator
    );

    // Dead address 0o4001 must not appear as a block (it is unreachable).
    assert!(!stream.blocks.contains_key(&0o4001), "dead block 0o4001 should not be decoded");

    compile_c(&CBackend::default().emit(&stream).unwrap());
}

/// A CA instruction followed by a TC loop: the frontend must decode both
/// instructions, the entry block must be present, and the generated C must
/// compile cleanly.
///
/// Placed at erasable 0o0100 to avoid TC 0o4000 → Go special case.
#[test]
fn c_ca_then_tc_compiles() {
    let mut img = blank_image();
    place(&mut img, 0o0100, &asm("CA 0o010"));   // load erasable[0o010] into A
    place(&mut img, 0o0101, &asm("TC 0o0100"));  // loop back to 0o0100

    let stream = decode(&img, 0o0100);

    assert!(stream.blocks.contains_key(&0o0100), "entry block 0o0100 missing");

    compile_c(&CBackend::default().emit(&stream).unwrap());
}

/// CCS (Count, Compare and Skip) must decode to a CcsBranch terminator on the
/// entry block, and the generated C (with its 4-way dispatch) must compile.
#[test]
fn c_ccs_four_way_branch_compiles() {
    let mut img = blank_image();
    // CCS 0o100 — requires EXTEND prefix; 2 words: EXTEND + CCS.
    place(&mut img, 0o4000, &asm("CCS 0o100"));
    // The 4 targets (positive, plus-zero, negative, minus-zero).
    for t in [0o4002u16, 0o4003, 0o4004, 0o4005] {
        place(&mut img, t, &asm(&format!("TCF 0o{t:o}")));
    }

    let stream = decode(&img, 0o4000);

    // Entry block must carry a CcsBranch terminator.
    assert!(
        matches!(stream.blocks[&0o4000].terminator, Terminator::CcsBranch(_)),
        "expected CcsBranch, got {:?}", stream.blocks[&0o4000].terminator
    );

    compile_c(&CBackend::default().emit(&stream).unwrap());
}

/// An empty stream (no blocks) must produce C that compiles cleanly.
#[test]
fn c_empty_stream_compiles() {
    let stream = InstrStream {
        blocks: std::collections::BTreeMap::new(),
        entry_points: vec![],
        indirect_targets: BTreeSet::new(),
    };
    compile_c(&CBackend::default().emit(&stream).unwrap());
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
/// must produce non-empty outputs.  The C output must compile cleanly.
#[test]
fn both_backends_accept_multi_instruction_program() {
    // Three sequential base instructions at erasable addresses.
    // Placed at 0o0100 to avoid TC 0o4000 → Go special case.
    let mut img = blank_image();
    place(&mut img, 0o0100, &asm("CA 0o010"));   // load
    place(&mut img, 0o0101, &asm("AD 0o010"));   // add mem[0o010] to A
    place(&mut img, 0o0102, &asm("TC 0o0100"));  // loop back

    let stream = decode(&img, 0o0100);

    let c    = CBackend::default().emit(&stream).unwrap();
    let wasm = make_wasm(&img, &[0o0100]);

    assert!(!c.is_empty(),    "C backend produced empty output");
    assert!(!wasm.is_empty(), "WASM backend produced empty output");
    validate_wasm(&wasm);
    compile_c(&c);
}

// ─── NDX (index) constant-folding tests ──────────────────────────────────────

/// NDX with an erasable operand cannot be constant-folded; the frontend must
/// fall back to an Indirect terminator.
///
/// NDX always requires EXTEND, so `asm("NDX 0o0010")` emits two words:
///   0o4000 = EXTEND
///   0o4001 = NDX word (K=0o0010, erasable → slicer returns None)
///   0o4002 = TCF 0o4004  (next instruction; its own block)
///   0o4004 = TCF 0o4004  (self-loop)
#[test]
fn c_ndx_erasable_falls_back_to_indirect() {
    let mut img = blank_image();
    // asm("NDX 0o0010") auto-prepends EXTEND → 2 words at 0o4000–0o4001.
    place(&mut img, 0o4000, &asm("NDX 0o0010"));
    // Next instruction after NDX is at 0o4002.
    place(&mut img, 0o4002, &asm("TCF 0o4004"));
    place(&mut img, 0o4004, &asm("TCF 0o4004"));

    // Indirect targets: the instruction following NDX and the self-loop target.
    let mut indirect_targets = BTreeSet::new();
    indirect_targets.insert(0o4002u16);
    indirect_targets.insert(0o4004u16);

    let stream = decode_stream(&img, &[0o4000], indirect_targets)
        .unwrap_or_else(|e| panic!("frontend error: {e}"));

    assert!(stream.blocks.contains_key(&0o4000), "entry block 0o4000 missing");
    let block = &stream.blocks[&0o4000];

    // Block contains EXTEND + NDX (both are InstrRecords).
    assert_eq!(
        block.instrs.len(), 2,
        "expected EXTEND + NDX in block, got {} instrs", block.instrs.len()
    );

    // Terminator must be Indirect (erasable K cannot be folded).
    assert!(
        matches!(block.terminator, Terminator::Indirect { .. }),
        "expected Indirect, got {:?}", block.terminator
    );

    compile_c(&CBackend::default().emit(&stream).unwrap());
}
