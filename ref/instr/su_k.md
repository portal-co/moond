# SU K — Subtract (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 28–35 (figs. 2-6..2-11; table 2-2).

Summary
- Operation: A := A - c(K) (with extend behavior handled by previous instructions in original AGC). Modernized version performs the subtraction as a single micro-op; carries/borrows and overflow produce `PINC` or `MINC` semantics which are documented here as helper actions.

Micro-op (C-like pseudocode)

```c
void SU_K(uint16_t K) {
    uint16_t z = Z;
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    B = G & 0x7FFF;
    P = parity(G);

    // Read the operand signed
    int16_t m = (int16_t)G;
    int32_t result = (int32_t)A - (int32_t)m; // compute with extra width

    // Update A with low 16 bits (two's complement semantics)
    A = (int16_t)(result & 0xFFFF);

    // Determine if increment/decrement of program counter carry is needed
    if (result > 0x7FFF) {
        // Overflow positive -> perform PINC (increment P register chain)
        schedule_PINC();
    } else if (result < -0x8000) {
        // Underflow -> perform MINC
        schedule_MINC();
    }

    // Advance to next instruction
    Z = z + 1;
    SQ = extract_order_code(B);
}
```

Notes
- This pseudocode uses 32-bit intermediate arithmetic to detect overflow relative to 16-bit signed range.
- The AGC's original "extend" behavior involving INCR/DECR pulses is represented by `schedule_PINC()`/`schedule_MINC()` calls which are documented elsewhere in the instruction set as carry/borrow propagation.

Citations
- AGCIS Issue 2, pp.28–35 (figs. 2-6..2-11, table 2-2).