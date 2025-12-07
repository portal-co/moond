# TCF F — Transfer Control to Fixed F (Block-2)

Summary
- Transfer control to a fixed F address (no change in register C). Used to jump to a fixed F bank address.

Pseudocode

void TCF_F(uint16_t F) {
    STMIC_stage();
    // Do not change C; set next instruction to F (Fixed area)
    S = F;
    SQ = extract_order_code(B);
    STD2_execute();
}

Notes
- Behavior matches AGC Issue 32 description; Block-2 notes to follow if differences are observed during full parsing.

Detailed behavior
- Effect: sets S = F (fixed-area fetch) while leaving C unchanged; SQ loaded from B; STD2_execute() finalizes by incrementing Z and calling forward.
- Subinstruction mapping: TCF F uses the TCF0 variant and STD2; when addressing extra-code bits (EXT), Special Instruction EXTEND must precede TCF F.

Block-2 differences (placeholder)
- Placeholder: add Block-2-specific semantics (banking, EXT handling) after review of the Block-2 instruction pages.
