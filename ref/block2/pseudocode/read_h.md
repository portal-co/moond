Read cycle (STMIC) — C-like pseudocode for memory read/write service

void STMIC_read(address_t addr) {
    // Standard memory inquiry cycle: RG, RB, RP etc. per AGCIS
    word_t word = MEM.read(addr);   // read into G register
    test_parity(word);              // TP -> alarm if incorrect
    // Gate into buffers B/P as needed by the instruction
}
