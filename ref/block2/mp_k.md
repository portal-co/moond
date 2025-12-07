# MP K — Multiply by K (Block-2)

Summary
- Multiply A by memory K producing a double-precision product in A (high) and L (low). Uses multi-step subinstructions; modernized into one routine.

Pseudocode

void MP_K(uint16_t K) {
    STMIC_stage();

    int32_t multiplicand = sign_extend15(read_memory(K));
    int32_t multiplier   = sign_extend15(A);

    int32_t product = multiplicand * multiplier; // result fits in 30 bits

    L = (uint16_t)(product & 0x7FFF);
    A = (uint16_t)((product >> 15) & 0x7FFF);

    // Set sign/overflow indicators as specified
    set_product_sign_and_overflow(product);

    B = I + 1;
    STD2_execute();
}

Notes
- This routine collapses the MP0/MP1/MP3 subinstruction sequence into a single logical operation for documentation and emulation clarity.