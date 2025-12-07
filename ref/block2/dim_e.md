# DIM E — Diminish E (Block-2)

Summary
- Decrease the magnitude of the quantity stored at E by one: positive values decrement, negative values increment (preserving sign). Used to reduce values by one without changing sign conventions.

Detailed pseudocode

void DIM_E(uint16_t E) {
    STMIC_stage();

    int32_t v = sign_extend15(read_memory(E));

    if (v > 0) {
        v = v - 1;
    } else if (v < 0) {
        v = v + 1;
    } else {
        // v == 0: remains zero
        v = 0;
    }

    write_memory(E, (uint16_t)(v & 0x7FFF));

    B = I + 1;
    STD2_execute();
}

Notes
- On E-memory writes, editing rules apply; if E represents special counters, Counter Priority Control may be notified on overflow/underflow.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
