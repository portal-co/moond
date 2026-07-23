//! Sparse direct-WASM plan regression coverage.

use std::collections::BTreeSet;

use agc_recompile::{
    backend::{wasm::WasmDirectBackend, DirectBackend},
    feed_direct_plan, plan_direct_functions,
};

fn place(image: &mut [u16; 4096], start: u16, words: &[u16]) {
    for (index, word) in words.iter().enumerate() {
        image[(start as usize + index) & 0x0fff] = *word;
    }
}

fn asm(line: &str) -> Vec<u16> {
    agc_asm::assemble(line).unwrap_or_else(|error| panic!("assemble {line:?}: {error}"))
}

#[test]
fn sparse_plan_emits_only_the_reachable_direct_functions() {
    let mut image = Box::new([0u16; 4096]);
    // 04000 jumps directly to the self-loop at 04002.  04001 is dead.
    place(&mut image, 0o4000, &asm("TC 0o4002"));
    place(&mut image, 0o4002, &asm("TC 0o4002"));

    let plan = plan_direct_functions(&image, &[0o4000], BTreeSet::new());
    assert_eq!(
        plan.functions.len(),
        2,
        "unexpected reachable plan: {plan:?}"
    );
    assert!(plan.functions.iter().all(|key| key.addr != 0o4001));

    let mut backend = WasmDirectBackend::<(), String>::new(vec![0o4000]);
    feed_direct_plan(&mut backend, &mut (), &image, &plan).unwrap();
    let bytes = backend.finish(&mut ()).unwrap();

    wasmparser::Validator::new()
        .validate_all(&bytes)
        .expect("sparse direct WASM must validate");

    let code_count = wasmparser::Parser::new(0)
        .parse_all(&bytes)
        .filter_map(Result::ok)
        .filter(|payload| matches!(payload, wasmparser::Payload::CodeSectionEntry(_)))
        .count();
    assert_eq!(code_count, plan.functions.len());
}
