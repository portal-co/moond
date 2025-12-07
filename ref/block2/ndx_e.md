# NDX E — Index Next Basic Instruction (Block-2)

Summary
- Indexing instruction: add content of location E to the next instruction (I+1) to derive the effective next instruction.

Pseudocode

void NDX_E(uint16_t E) {
    // Standard memory inquiry for E
    STMIC_stage();

    // Fetch indexing quantity (E) and the instruction at I+1
    int16_t idx = sign_extend15(read_memory(E));
    Instruction next_inst = fetch_instruction(I + 1); // returns (order_code, address, raw_word)

    // Derive new instruction by adding idx to the word/address of next_inst
    // derive_instruction implements AGC indexing rules: add low 10/12 bits to address portion,
    // handle EXT bit/quarter codes, wrap/carry, and adjust order code if overflow into opcode bits
    Instruction derived = derive_instruction(next_inst, idx);

    // Place derived instruction as the next to execute (B/S/SQ loaded as in AGC)
    B = derived.raw_word;
    S = derived.address;
    SQ = derived.order_code;

    // Finalize and call forward
    STD2_execute();
}

Helpers
- derive_instruction(next_inst, idx): performs bitwise addition of idx to the instruction word/address per AGC rules, preserves EXT semantics, and returns normalized Basic Instruction (not an Extra-Code instruction unless valid).

Notes
- NDX E collapses NDXO/NDXI subinstructions into one logical operation for documentation; preserve precise bit/quarter-code arithmetic in derive_instruction for emulation fidelity.