# AD K — Add memory into A (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 31–33 (sections 2-33..2-35, figure 2-9).

Summary
- Operation: Perform arithmetic `A := A + [K]`. On overflow/underflow, schedule increment/decrement of the overflow counter (PINC/MINC semantics).
- Modernization: Single micro-op that performs the add and signals overflow actions via helper calls.

Pseudocode

```c
// AD K: Add memory into accumulator
// See ref/definitions/STD2.md for canonical subinstruction completion
void AD_K(uint16_t K) {
    // Fetch operand from memory address K
    uint16_t operand = memory[K];
    
    // Perform signed 15-bit addition
    int32_t a = sign_extend15(A);
    int32_t k = sign_extend15(operand);
    int32_t sum = a + k;

    // Store result (low 15 bits) into accumulator
    A = (uint16_t)(sum & 0x7FFF);
    
    // Set overflow flags if sum exceeds 15-bit range
    // On overflow: schedule PINC (positive overflow)
    // On underflow: schedule MINC (negative overflow)
    if (sum > 0x3FFF || sum < -0x4000) {
        set_overflow_flags(sum);
    }

    // Standard instruction completion (STD2 inline)
    // See ref/definitions/STD2.md
    Z = Z + 1;                          // Increment program counter
    uint16_t next = memory[Z];          // Fetch next instruction
    SQ = extract_order_code(next);      // Decode operation
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

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
