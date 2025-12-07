# AUG E — Augment E (Block-2)

Summary
- Increase the magnitude of the quantity stored at E by one: positive values increment, negative values decrement (preserving sign). Useful for corner-case adjustments (angular resolution increments).

Detailed pseudocode

void AUG_E(uint16_t E) {
    STMIC_stage();

    int32_t v = sign_extend15(read_memory(E));

    if (v > 0) {
        v = v + 1; // increment magnitude for positive values
    } else if (v < 0) {
        v = v - 1; // decrement magnitude for negative values (more negative)
    } else {
        // v == 0: define as increment to +1 (matches AGCIS semantics for AUG when zero?)
        v = 1;
    }

    write_memory(E, (uint16_t)(v & 0x7FFF));

    B = I + 1;
    STD2_execute();
}

Notes
- Implementation must handle E-memory edits on write and may trigger Counter Priority Control if E addresses counters.