# MSU E — Modular Subtract E (Block-2)

Summary
- Compute modular difference of cyclic TWO's complement numbers in A and E (useful for angular differences); result in A.

Detailed pseudocode

void MSU_E(uint16_t E) {
    // Standard memory inquiry
    STMIC_stage();

    // Read operands (A already contains minuend; E supplies subtrahend)
    uint16_t minuend = A;
    uint16_t subtrahend = read_memory(E); // handles E-memory edit/restore

    // Compute modular TWO's-complement difference (angle subtraction semantics)
    uint16_t result = twos_modular_subtract(minuend, subtrahend);
    A = result;

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- `twos_modular_subtract` performs cyclic subtraction with final sign correction as described in AGCIS (ensures result is expressed in ONE's complement convention used by AGC for angular values).
- Use helpers to preserve exact bit and sign behaviors when converting between TWO's- and ONE's-complement representations.