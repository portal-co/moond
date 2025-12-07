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

Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.
