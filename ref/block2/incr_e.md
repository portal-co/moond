# INCR E — Increment E (Block-2)

Summary
- Increment location E by one (useful for counters and state variables). Handles E-memory edit/restore rules and signals Counter Priority Control when counter addresses are involved.

Detailed pseudocode

void INCR_E(uint16_t E) {
    // Standard memory inquiry
    STMIC_stage();

    // Read, increment, and write back (E-memory edit/restore handled by helpers)
    int32_t v = sign_extend15(read_memory(E));

    // Increment magnitude by one (overflow bit is lost on writes to E as specified)
    v = v + 1;

    write_memory(E, (uint16_t)(v & 0x7FFF));

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- If E corresponds to a counter address that requires Counter Priority handling, the implementation must notify the Counter Priority Control on overflow as described in AGCIS (e.g., addresses 0024..0027)."
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
