# MP K — Multiply (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 46–60 (figs. 2-17..2-25; table 2-4).

Summary
- Operation: Multiply A by c(K) and accumulate into the `A`/`Q` pair using a shift-and-add algorithm. Modernized version exposes the algorithm as C-like pseudocode suitable for emulation.

Micro-op (C-like pseudocode)

```c
// Multiply: A:Q <- A * c(K) (A high, Q low) using shift-and-add
void MP_K(uint16_t K) {
    uint16_t z = Z;
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    uint16_t operand = G & 0x7FFF; // 15-bit magnitude

    // Use signed 16-bit values for A and operand
    int16_t a = (int16_t)A;
    int16_t m = (int16_t)operand;

    int32_t product = (int32_t)a * (int32_t)m; // 32-bit result

    // Place high 16 bits into A, low 16 bits into Q
    A = (uint16_t)((product >> 16) & 0xFFFF);
    Q = (uint16_t)(product & 0xFFFF);

    // PC advance
    Z = z + 1;
    SQ = extract_order_code(G & 0x7FFF);
}
```

Notes
- The AGC's microcoded multiply used repeated shifts and adds across multiple cycles; for documentation and emulation readability we present a single-step multiply using a full-width intermediate type.
- Sign extension semantics follow two's complement convention; `A:Q` layout mirrors original AGC (A holds high-order bits).

Citations
- AGCIS Issue 2, pp.46–60 (figs. 2-17..2-25; table 2-4).