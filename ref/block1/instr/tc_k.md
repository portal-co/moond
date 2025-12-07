# TC K — Transfer Control (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 18–19 (sections 2-16..2-18).

Summary
- Operation: Save the current next-address (`Z`) into `Q` and set the program counter `Z` to `K + 1`, then begin execution at the order code fetched from `K`.
- Modernization: Presented as a single micro-op routine (no subinstructions). Octal constants use `0o` prefix.

Micro-op (C-like pseudocode)

```c
void TC_K(uint16_t K) {
    uint16_t z = Z;            // next address

    // STMIC: stage memory inquiry for K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    B = G & 0x7FFF;
    P = parity(G);

    // Save return address and set new PC
    Q = z;
    Z = (uint16_t)(B + 1);

    // Load next order code
    SQ = extract_order_code(B);
}
```

Citations
- AGCIS Issue 2, pp.18–19, §§2-16–2-18.

Notes
- The original AGC implements this with `TC0` + `STD2` subinstructions; we inline the behavior so the instruction appears atomic for emulator documentation.

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
