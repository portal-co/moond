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