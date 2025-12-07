# TCSAJ K — Transfer Control to Specified Address K (Peripheral) (Block-2)

Summary
- Peripheral-supplied Transfer Control: GSE supplies address K; TCSAJ K loads that address into S and STD2 finalizes fetch (used in ground test and peripherals).

Detailed pseudocode

void TCSAJ_K(uint16_t K_from_gse) {
    // TCSAJ is typically invoked with K supplied by GSE; model as direct placement
    S = K_from_gse;
    // STD2 will fetch the instruction at K and call forward
    STD2_execute();
}

Notes
- For interactive GSE testing, FETCH/STORE interactions can be modeled by fetch_instruction_via_S and related helpers.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
