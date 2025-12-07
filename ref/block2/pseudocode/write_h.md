Write cycle (STMIC) — C-like pseudocode for memory write service

void STMIC_write(address_t addr, word_t value) {
    // Prepare parity bit and write via WAs into E memory
    word_t with_parity = set_parity_bit(value);
    MEM.write(addr, with_parity);
}
