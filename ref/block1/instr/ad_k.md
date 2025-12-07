# AD K — Add memory into A (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 31–33 (sections 2-33..2-35, figure 2-9).

Summary
- Operation: Perform arithmetic `A := A + [K]`. On overflow/underflow, schedule increment/decrement of the overflow counter (PINC/MINC semantics).
- Modernization: Single micro-op that performs the add and signals overflow actions via helper calls.

Micro-op (C-like pseudocode)

```c
// Canonicalized AD_K using helpers from ref/cpu/registers.md and ref/Instruction.md
void AD_K(uint16_t K) {
    // Standard memory inquiry and operand read
    STMIC_stage();

    int32_t a = sign_extend15(A);
    int32_t k = sign_extend15(read_memory(K));

    int32_t sum = a + k;

    // Store result (low 15 bits) and set overflow indicators via helper
    A = (uint16_t)(sum & 0x7FFF);
    set_add_overflow_flags(sum); // sets PINC/MINC as required (TODO:VERIFY exact bit mapping)

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}
```

Citations
- AGCIS Issue 2, pp.31–33, §§2-33–2-35 and figure 2-9.

Notes
- `schedule_PINC()` / `schedule_MINC()` are documented elsewhere (these emulate increment/decrement of the overflow-counter chain as in AGC hardware).

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.
