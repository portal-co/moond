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

Detailed pseudocode

void TCF_F(uint16_t F) {
    // Standard memory inquiry
    STMIC_stage();

    // Do not change C; set next instruction pointer to fixed F
    S = F;
    SQ = extract_order_code(B); // EXT handling should be done via EXTEND helper

    // STD2 finalization
    STD2_execute();
}

Notes
- TCF_F leaves C unchanged and uses the fixed-area address in F for the next fetch; model as atomic with STD2_execute() encapsulating final pulses.
- Block-2 differences (placeholder): record any Block-2-specific bank/EXT semantics when discovered.
