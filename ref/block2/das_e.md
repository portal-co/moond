# DAS E — Double Add to Storage E (Block-2)

Summary
- Add the double-precision quantity in A,L to the double-precision quantity at memory locations E and E+1, store the low/hi parts appropriately, and encode any net overflow into A.

Detailed pseudocode

void DAS_E(uint16_t E) {
    // Standard memory inquiry (E and E+1 may be read in sequence)
    STMIC_stage();

    int32_t a_hi = sign_extend15(A);
    int32_t a_lo = sign_extend15(L);

    // Read double-precision memory pair (E, E+1)
    int32_t e_lo = sign_extend15(read_memory(E));
    int32_t e_hi = sign_extend15(read_memory(E + 1));

    // Compose 30-bit signed quantities
    int64_t a_full = ((int64_t)a_hi << 15) | (a_lo & 0x7FFF);
    int64_t e_full = ((int64_t)e_hi << 15) | (e_lo & 0x7FFF);

    int64_t sum = a_full + e_full;

    // Store low/high parts into E and E+1 (without overflow bit per AGC storing rules)
    write_memory(E + 1, (uint16_t)((sum >> 15) & 0x7FFF));
    write_memory(E, (uint16_t)(sum & 0x7FFF));

    // Encode net overflow into A: +1 -> 000001, -1 -> 177776, none -> sum_high (low 15 bits placed in A)
    A = encode_double_add_overflow(sum);

    B = I + 1;
    STD2_execute();
}

Notes
- encode_double_add_overflow(sum) implements AGC's rule for placing overflow indicators into A when double-precision storage overflows occur; helpers must preserve AGC bit semantics and editing for E-memory writes.