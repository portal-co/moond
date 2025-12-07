# MP K — Multiply (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 46–60 (figs. 2-17..2-25; table 2-4).

Summary
- Operation: Multiply A by c(K) and accumulate into the `A`/`Q` pair using a shift-and-add algorithm. Modernized version exposes the algorithm as C-like pseudocode suitable for emulation.

Micro-op (C-like pseudocode)

```c
// Canonicalized MP_K: present logical effect; small helpers used for I/O and sign handling
void MP_K(uint16_t K) {
    // Standard memory inquiry and operand fetch
    STMIC_stage();

    int32_t multiplicand = sign_extend15(read_memory(K));
    int32_t multiplier   = sign_extend15(A);

    int32_t full_product = multiplicand * multiplier; // logical 30-bit product

    // Store into A (high) and L/Q (low) using canonical 15-bit fields
    L = (uint16_t)(full_product & 0x7FFF);
    A = (uint16_t)((full_product >> 15) & 0x7FFF);

    set_product_sign_and_overflow(full_product); // TODO:VERIFY exact overflow bit rules

    B = I + 1;
    STD2_execute();
}
```

Notes
- The AGC's microcoded multiply used repeated shifts and adds across multiple cycles; for documentation and emulation readability we present a single-step multiply using a full-width intermediate type.
- Sign extension semantics follow two's complement convention; `A:Q` layout mirrors original AGC (A holds high-order bits).

Citations
- AGCIS Issue 2, pp.46–60 (figs. 2-17..2-25; table 2-4).