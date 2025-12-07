# XCH K — Exchange A with memory (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 20–22 (sections 2-19..2-21).

Summary
- Operation: Exchange the contents of accumulator `A` with the word at memory address `K`.
- Modernization: Presented as a single micro-op. Parity and sign bits preserved according to AGC conventions.

Micro-op (C-like pseudocode)

```c
void XCH_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC: fetch memory at K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    uint16_t mem = G & 0xFFFF;

    // Exchange A and memory
    uint16_t tmp = A;
    A = mem;
    MEM[S] = tmp;

    // Update parity and ordercode staging
    P = parity(A);
    B = mem & 0x7FFF;
    SQ = extract_order_code(B);

    // Advance PC
    Z = z + 1;
}
```

Citations
- AGCIS Issue 2, pp.20–22, §§2-19–2-21.

Notes
- The hardware performs this as multiple microcycles; we present the logical exchange suitable for emulation and documentation.

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
