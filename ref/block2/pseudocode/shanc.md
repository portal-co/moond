SHANC — C-like pseudocode (Shift and Add One)

/* Shift left content of addressed counter and add one; used for serial-to-parallel conversion. */
void SHANC(address_t ctr_addr) {
    word_t e = MEM.read(ctr_addr);
    test_parity(e);
    word_t res = ((e << 1) + 1) & WORD_MASK;
    MEM.write(ctr_addr, res);
    if (test_bit(e,15)) signal_UPRUPT();
    if (detect_overflow_shift(e, res)) { reverse_sign_bit(res); prevent_end_around_carry(); }
}
