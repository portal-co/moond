# SU E — Subtract E from A (Block-2)

Summary
- Subtract content of memory location E from register A and store the difference in A (A = A - c(E)).

Detailed pseudocode

void SU_E(uint16_t E) {
    // Standard memory inquiry and read
    STMIC_stage();

    int32_t a = sign_extend15(A);
    int32_t e = sign_extend15(read_memory(E));

    int32_t diff = a - e;

    // Store result as 15-bit value and set overflow flags per AGC rules
    A = (uint16_t)(diff & 0x7FFF);
    set_sub_overflow_flags(diff);

    B = I + 1;
    STD2_execute();
}

Notes
- This collapses the SU0/STD2 subinstruction behavior; helpers maintain AGC-specific overflow and sign behavior.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
