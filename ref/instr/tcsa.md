# TCSA — Test Set Start At Specified Address (modernized)

Source: `agcis_2_machine_instructions.pdf` — p.100 (section 2‑124).

Summary
- Operation: Start execution at a specified address `SA` entered via the Computer Test Set. Functionally similar to `GO` but uses the test-set supplied start address.

Micro-op (C-like pseudocode)

```c
void TCSA(uint16_t SA) {
    // Use the address provided by the test set
    Z = SA;
    B = MEM[SA] & 0x7FFF;
    SQ = extract_order_code(B);
}
```

Citations
- AGCIS Issue 2, p.100 (section 2‑124).

Notes
- For emulator usage, `SA` is obtained from the test-set I/O interface.