SHINC — C-like pseudocode (shift content of addressed counter)

void SHINC(address_t ctr_addr) {
    word_t e = MEM.read(ctr_addr);
    // Shift left by 1: c(CTR) = 2 * b(CTR)
    word_t shifted = (e << 1) & WORD_MASK;
    MEM.write(ctr_addr, shifted);

    // If bit15 (original bit 15) == 1, signal UPRUPT to program interrupt control
    if (test_bit(e, 15)) signal_UPRUPT();

    // On overflow, reverse bit 16 and prevent end-around carry (AGCIS behavior)
    if (detect_overflow_shift(e, shifted)) handle_shift_overflow(ctr_addr);
}
