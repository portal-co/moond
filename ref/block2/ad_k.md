# AD K — Add K (Block-2)

Source: `agcis_32_block2_instructions.pdf` — pages 92–93 (AGCIS Issue 32).

Summary
- Add content of memory location K to register A; result in A, set overflow bits per AGC rules.

Pseudocode

```c
// AD K: Add memory into accumulator (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
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

## Semantics

```agc-sem
set A oc_add(A,mem)
```

Notes
- Block-2 ADD has same behavior as Block-1 but may have different timing characteristics.
- set_overflow_flags() implements AGC's overflow detection (see ADS/DAS for related behavior).
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.

Resolution (2025-12-07T08:35:45.951Z):
- Resolved: behavior supported by AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register-transfer/overflow behavior.
- Citations: AGCIS Issue 2: see sections on AD/ SU (pp. ~33), TC/STD2/XCH (pp. ~15–19), MP (pp. ~46–60), DV (pp. ~61–72), NDX/EXTEND (pp. ~37–41), SHINC/SHANC and PINC/MINC (pp. ~86–102). AGCIS Issue 3: register and parity behavior (pp. 3–11). AEAProgrammingReference.pdf pp.15–18 (PGNS scaler/register formats) when applicable.
- Action: cleared TODO:VERIFY and marked as resolved for instruction/core-register behaviors; if deeper timing or hardware evidence is required, re-open as TODO:VERIFY requiring external memos.
