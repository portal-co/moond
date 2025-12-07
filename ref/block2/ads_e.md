# ADS E — Add to Storage E (Block-2)

Summary
- Add the content of register A to storage location E and store the sum in both A (with overflow bit) and E (without overflow bit). Useful for accumulating results into memory with overflow reporting in A.

Detailed pseudocode

void ADS_E(uint16_t E) {
    // Standard memory inquiry
    STMIC_stage();

    int32_t a = sign_extend15(A);
    int32_t e = sign_extend15(read_memory(E));

    int32_t sum = a + e;

    // Store sum in E without overflow-bit (edited write)
    write_memory(E, (uint16_t)(sum & 0x7FFF));

    // Store A with overflow bit if present (AGC specific encoding)
    A = encode_with_overflow(sum);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- encode_with_overflow(sum) returns the 15/16-bit representation put into A where positive/negative overflow are encoded as 000001 and 177776 respectively per AGC conventions.