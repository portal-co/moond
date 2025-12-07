PINC — C-like pseudocode (increment overflow counter)

/* Increment content of addressed counter by one (used for OVCTR). */
void PINC(address_t ctr_addr) {
    word_t val = MEM.read(ctr_addr);
    word_t inc = (val + 1) & WORD_MASK;
    MEM.write(ctr_addr, inc);
    if (inc == 0) {
        // Overflow happened; send signal to Priority Control per AGCIS
        signal_counter_overflow();
    }
}
