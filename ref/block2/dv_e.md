# DV E — Divide by E (Block-2)

Summary
- Divide double-precision quantity (A,L) by single-precision divisor in E; writes quotient in A and remainder in L. Complex multi-subinstruction sequence is presented as one routine.

Pseudocode

void DV_E(uint16_t E) {
    STMIC_stage();

    int32_t dividend = ((int32_t)A << 15) | (int32_t)L; // sign/format per AGC conventions
    int16_t divisor  = sign_extend15(read_memory(E));

    if (divisor == 0) {
        // Division-by-zero handling per AGCIS: set states, request RUPT or defined behavior
        handle_divide_by_zero();
        return;
    }

    int32_t quotient  = dividend / divisor;
    int32_t remainder = dividend % divisor;

    A = (uint16_t)(quotient & 0x7FFF);
    L = (uint16_t)(remainder & 0x7FFF);

    set_div_sign_and_overflow(quotient, remainder);

    B = I + 1;
    STD2_execute();
}

Notes
- The multi-action DV0..DV7 sequence is represented by a single function for clarity; helpers encapsulate low-level bit- and timing-sensitive behaviors.