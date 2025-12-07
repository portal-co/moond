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
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.
