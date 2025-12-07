# Central Processor — Overview (modernized)

Source: `agcis_3_central_processor.pdf` — pp.5–8, fig. 3-1.

Summary
- Describes the Central Processor (CP): registers A, Q, Z, LP, B, G; the Adder (X/Y); 16 Write Amplifiers (WA01–WA16); parity block; service gates; and the BNK register.
- Documentation style: C-like pseudocode primitives model register & memory operations, timing and control pulses are described as logical side-effects (not cycle-level pulses).

Concepts and conventions used in this CPU doc set
- Word layout: 16 bits total — bits 1..15 are the value bits; bit 16 is the sign bit; stick 15 is used for parity/overflow in AGC mapping. We use `(u)int15_t` and `int16_t` for clarity.
- Write/read/clear primitives are modeled as atomic helpers: `write_register(reg, value)`, `read_register(reg)`, `set_bitstick(stick, value)`, `compute_parity_bit(value)`.
- Where the AGC uses address-dependent write gating (e.g., `WG1G..WG6G`), we represent these as logical helpers such as `gated_write_G(address, value, mode)`.

Block diagram notes
- The CP is constructed from 16 identical bit-sticks; each stick contains flip-flops for each register bit, one adder section, a write amplifier, and service gates. Sticks 1–14 store bits 1–14; stick 15 holds parity/overflow; stick 16 holds the sign bit (pp.5–6).
- The G register is the memory buffer between the CP and main memory (pp.9–12). Write amplifier outputs route to G and are gated by address-dependent signals to place bits into the correct bit positions.

Example (C-like primitives)

```c
// Read a word from memory into the G register (modeled)
void fetch_into_G(uint16_t addr) {
    uint16_t word = MEM[addr] & 0xFFFF;
    write_register("G", word);
}

// Stage-and-fetch conceptual helper used by instruction pseudocode
void STMIC_stage(uint16_t k_addr) {
    uint16_t z = read_register("Z");
    // stage S/Y/X as in AGC
    write_register("S", z);
    write_register("Y", z);
    write_register("X", 0);
    if (z >= 0o20) fetch_into_G(z);
    // stage B and parity for next-order word
    write_register("B", read_register("G") & 0x7FFF);
    write_register("P", compute_parity_bit(read_register("G") & 0x7FFF));
}
```

Citations
- AGCIS Issue 3, pp.5–12, figs. 3‑1..3‑3 (Central Processor, register operation, G register).