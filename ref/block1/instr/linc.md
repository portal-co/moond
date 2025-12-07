# LINC — Load Increment (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.101–102, fig. 2‑43.

Summary
- Operation: Write into the addressed location the data provided by the Computer Test Set (used for manual loading of memory during testing). The address and data are supplied via the test-set keyboard.

Micro-op (C-like pseudocode)

```c
void LINC(uint16_t K, uint16_t data) {
    // Write the test-set supplied data into memory
    MEM[K] = data | (compute_parity_bit(data) << 15);

    // Advance normally
    Z = Z + 1;
    SQ = extract_order_code(MEM[Z] & 0x7FFF);
}
```

Notes
- In practice `LINC` reads `data` from the keyboard/test-set interface before writing.

Citations
- AGCIS Issue 2, pp.101–102, fig. 2‑43.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
