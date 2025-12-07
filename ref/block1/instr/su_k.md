# SU K — Subtract (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 28–35 (figs. 2-6..2-11; table 2-2).

Summary
- Operation: A := A - c(K) (with extend behavior handled by previous instructions in original AGC). Modernized version performs the subtraction as a single micro-op; carries/borrows and overflow produce `PINC` or `MINC` semantics which are documented here as helper actions.

Micro-op (C-like pseudocode)

```c
// Canonicalized SU_K using helpers
void SU_K(uint16_t K) {
    STMIC_stage();

    int32_t a = sign_extend15(A);
    int32_t k = sign_extend15(read_memory(K));

    int32_t result = a - k;

    // Store and set overflow helpers
    A = (uint16_t)(result & 0x7FFF);
    set_sub_overflow_flags(result); // TODO:VERIFY exact PINC/MINC mapping

    B = I + 1;
    STD2_execute();
}
```

Notes
- This pseudocode uses 32-bit intermediate arithmetic to detect overflow relative to 16-bit signed range.
- The AGC's original "extend" behavior involving INCR/DECR pulses is represented by `schedule_PINC()`/`schedule_MINC()` calls which are documented elsewhere in the instruction set as carry/borrow propagation.

Citations
- AGCIS Issue 2, pp.28–35 (figs. 2-6..2-11, table 2-2).
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
