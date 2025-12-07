# MSU E — Modular Subtract E (Block-2)

Summary
- Compute modular difference of cyclic TWO's complement numbers in A and E (useful for angular differences); result in A.

Pseudocode

void MSU_E(uint16_t E) {
    STMIC_stage();

    // cyclic TWO's-complement difference
    uint16_t a = A;
    uint16_t e = read_memory(E);

    uint16_t result = twos_modular_subtract(a, e);
    A = result;

    B = I + 1;
    STD2_execute();
}

Notes
- `twos_modular_subtract` implements the cyclic wrap-around and final sign correction described in AGCIS; this is important for angle arithmetic.