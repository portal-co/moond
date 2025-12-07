# CSK K — Complement & Skip (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 22–23 (sections 2-22..2-24).

Summary
- Operation: Load the complemented value of memory at `K` into accumulator `A` (A := ~[K]), then advance to the next instruction.
- Modernization: Presented as a single micro-op; parity handling is preserved.

Micro-op (C-like pseudocode)

```c
void CSK_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC: fetch memory at K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];

    // Complement the fetched word (bitwise complement on 16 bits)
    uint16_t val = G & 0xFFFF;
    A = (uint16_t)(~val & 0xFFFF);

    // Update parity and staging
    P = parity(A);
    B = G & 0x7FFF;
    SQ = extract_order_code(B);

    // Advance PC
    Z = z + 1;
}
```

Citations
- AGCIS Issue 2, pp.22–23, §§2-22–2-24.

Notes
- The original AGC name in the combined file was `CSK` (Clear & Complement variant). This file uses the mnemonic `CSK` to match the original doc.

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
