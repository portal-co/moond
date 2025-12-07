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

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.
