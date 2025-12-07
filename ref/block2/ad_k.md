# AD K — Add K (Block-2)

Summary
- Add content of memory location K to register A; result in A, set overflow bits per AGC rules.

Detailed pseudocode

void AD_K(uint16_t K) {
    // Standard memory inquiry
    STMIC_stage();

    // Read K and perform signed 15-bit add
    int32_t a = sign_extend15(A);
    int32_t k = sign_extend15(read_memory(K));

    int32_t sum = a + k;

    // Store result (low 15 bits) into A and set overflow per AGC semantics
    A = (uint16_t)(sum & 0x7FFF);
    set_add_overflow_flags(sum);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- set_add_overflow_flags(sum) should implement AGC's overflow detection that sets sign/overflow flip-flops and encodes +1/-1 into A where AGC specifies (see ADS/DAS for related behavior)."