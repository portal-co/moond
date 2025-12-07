# Parity Block — Generation and checking (modernized)

Source: `agcis_3_central_processor.pdf` — pp.29–33, figs. 3-11, tables 3-6..3-7.

Summary
- The parity block computes parity for words read from memory (addresses >= 0o30) and sets the parity bit (bit 0 / stick 15) in register `G`. A parity alarm (`PAL`) is asserted on mismatch.
- The block uses a small tree of flip-flops A–E and logic gates to compute parity (pp.29–31). Table 3‑6 and 3‑7 define the parity-tree outputs and parity bit generation rules.

Pseudocode primitives

```c
// Compute parity bit used in AGC memory words
// Returns parity bit (0 or 1) where 1 indicates odd number of ones in the value bits
uint8_t compute_parity_bit(uint16_t word15) {
    // word15 is lower 15 value bits (bits 1..15)
    // AGC parity is 1 if number of ONEs is odd (ignoring stored parity bit)
    uint32_t v = word15 & 0x7FFF;
    uint8_t count = (uint8_t)__builtin_popcount(v);
    return (count & 1U) ? 1 : 0;
}

// When reading a memory word, verify parity
void check_memory_parity(uint16_t addr) {
    uint16_t word = MEM[addr] & 0xFFFF;
    uint8_t stored_parity = (word >> 15) & 1U; // bit 0 / stick 15
    uint8_t computed = compute_parity_bit(word & 0x7FFF);
    if (stored_parity != computed) {
        signal_parity_alarm(addr);
    }
}
```

Notes
- The AGC implements the parity tree in hardware (flip-flops A..E feeding gates A..P). The helper `__builtin_popcount` is used here as a concise emulation of the parity tree operation.

Citations
- AGCIS Issue 3, pp.29–33, figs. 3‑11 and tables 3‑6..3‑7.