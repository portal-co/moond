# NDX E — Index Next Basic Instruction (Block-2)

Summary
- Indexing instruction: add content of location E to the next instruction (I+1) to derive the effective next instruction.

Pseudocode

void NDX_E(uint16_t E) {
    STMIC_stage();

    uint16_t idx = read_memory(E);
    Instruction next = fetch_instruction(I + 1);

    Instruction derived = derive_instruction(next, idx);

    // Place derived instruction into B/S/SQ as next to execute
    B = derived.raw_word;
    S = derived.address;
    SQ = derived.order_code;

    STD2_execute();
}

Notes
- `derive_instruction` implements the addition rules described in AGCIS (including wrap/limits and EXT handling). Subinstructions NDXX0/NDXX1 are represented by the single `derive_instruction` step.