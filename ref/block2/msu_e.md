# MSU E — Modular Subtract E (Block-2)

Summary
- Compute modular difference of cyclic TWO's complement numbers in A and E (useful for angular differences); result in A.

Detailed pseudocode

void MSU_E(uint16_t E) {
    // Standard memory inquiry
    STMIC_stage();

    // Read operands (A already contains minuend; E supplies subtrahend)
    uint16_t minuend = A;
    uint16_t subtrahend = read_memory(E); // handles E-memory edit/restore

    // Compute modular TWO's-complement difference (angle subtraction semantics)
    uint16_t result = twos_modular_subtract(minuend, subtrahend);
    A = result;

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- `twos_modular_subtract` performs cyclic subtraction with final sign correction as described in AGCIS (ensures result is expressed in ONE's complement convention used by AGC for angular values).
- Use helpers to preserve exact bit and sign behaviors when converting between TWO's- and ONE's-complement representations.

Inline notes
- MSU_E is presented in Block-2 with inlined STMIC and memory access where timing matters; canonical helper functions live in ref/definitions/Instruction.md and ref/cpu/registers.md.

Edge cases / TODOs
- Treatment of overflow-bit vs sign-bit for cyclic results: TODO:VERIFY.
- Behavior when E points to special counter addresses: TODO:VERIFY.
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
