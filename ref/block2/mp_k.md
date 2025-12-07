# MP K — Multiply by K (Block-2)

Summary
- Multiply A by memory K producing a double-precision product in A (high) and L (low). Uses multi-step subinstructions; modernized into one routine.

Pseudocode

void MP_K(uint16_t K) {
    // Stage 0: standard memory inquiry for K
    STMIC_stage();

    // Read multiplicand (K) and multiplier (A) as signed 15-bit values
    int32_t multiplicand = sign_extend15(read_memory(K));
    int32_t multiplier   = sign_extend15(A);

    // Perform the multiplication (implemented by MP0/MP1/MP3 subinstruction sequence on AGC)
    // Use full 30-bit two's-complement arithmetic to obtain exact product
    int32_t full_product = multiplicand * multiplier; // fits in signed 30 bits

    // Store low and high 15-bit parts into L (low) and A (high) as AGC does
    L = (uint16_t)(full_product & 0x7FFF);                     // low 15 bits
    A = (uint16_t)((full_product >> 15) & 0x7FFF);             // high 15 bits

    // Set sign/overflow indicators per AGC conventions (helper encapsulates bit rules)
    set_product_sign_and_overflow(full_product);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- This pseudocode represents the logical effect of the MP0/MP1/MP3 subinstruction sequence; helpers preserve AGC-specific overflow and sign-bit semantics.