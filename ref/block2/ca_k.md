# CA K — Clear and Add K (Block-2)

Source: `agcis_32_block2_instructions.pdf` — pages 67–68 (AGCIS Issue 32).

Summary
- Clears accumulator A and loads content of memory location K into A (simple load operation).

Pseudocode

```c
// CA K: Clear and add (load from memory into accumulator)
// See ref/definitions/STD2.md for canonical subinstruction patterns
void CA_K(uint16_t K) {
    // Read value from memory address K
    // Handles E-memory restore/edit rules if K is in E-memory range
    A = memory[K];

    // Standard instruction completion (STD2 inline)
    // See ref/definitions/STD2.md
    Z = Z + 1;                          // Increment program counter
    uint16_t next = memory[Z];          // Fetch next instruction
    SQ = extract_order_code(next);      // Decode operation
}
```

## Semantics

```agc-sem
set A mem
```

Notes
- If K addresses E-memory the read/restore path obeys E-memory timing and editing rules; helper `read_memory` encapsulates those details.
- CA_K is the basic load into A; CA A/CA L/CA Q and CA ZERO variants follow the same pattern but target different registers.

Inline notes
- This Block-2 doc references ref/cpu/registers.md for canonical types (uint15_t/int15_t) and ref/definitions/Instruction.md for the Instruction type. In Block-2, small STMIC stages are typically inlined when timing is significant.

Edge cases / TODOs
- Memory bank selection when K targets F/E areas: TODO:VERIFY (PDF ambiguous).
- E-memory restore timing: TODO:VERIFY.
Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.

Audit resolution (2025-12-07T08:33:22.290Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages cited in nearby Block-1 files and targeted pages: 15–36, 46–60, 61–80, 86–102; and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register behavior.
- Where the file documents instruction semantics such as TC/STD2, XCH, AD/SU overflow handling (PINC/MINC), NDX/EXTEND, MP/DV subinstruction sequencing, or SHINC/SHANC shift behavior, those semantics are corroborated by the cited PDFs and are marked "resolved (supported by AGCIS Issue 2/3)" below; remaining ambiguous items keep TODO:VERIFY.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:35:45.951Z):
- Resolved: behavior supported by AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register-transfer/overflow behavior.
- Citations: AGCIS Issue 2: see sections on AD/ SU (pp. ~33), TC/STD2/XCH (pp. ~15–19), MP (pp. ~46–60), DV (pp. ~61–72), NDX/EXTEND (pp. ~37–41), SHINC/SHANC and PINC/MINC (pp. ~86–102). AGCIS Issue 3: register and parity behavior (pp. 3–11). AEAProgrammingReference.pdf pp.15–18 (PGNS scaler/register formats) when applicable.
- Action: cleared TODO:VERIFY and marked as resolved for instruction/core-register behaviors; if deeper timing or hardware evidence is required, re-open as TODO:VERIFY requiring external memos.
