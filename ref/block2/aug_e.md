# AUG E — Augment E (Block-2)

Summary
- Increase the magnitude of the quantity stored at E by one: positive values increment, negative values decrement (preserving sign). Useful for corner-case adjustments (angular resolution increments).

Detailed pseudocode

void AUG_E(uint16_t E) {
    STMIC_stage();

    int32_t v = sign_extend15(read_memory(E));

    if (v > 0) {
        v = v + 1; // increment magnitude for positive values
    } else if (v < 0) {
        v = v - 1; // decrement magnitude for negative values (more negative)
    } else {
        // v == 0: define as increment to +1 (matches AGCIS semantics for AUG when zero?)
        v = 1;
    }

    write_memory(E, (uint16_t)(v & 0x7FFF));

    B = I + 1;
    STD2_execute();
}

Notes
- Implementation must handle E-memory edits on write and may trigger Counter Priority Control if E addresses counters.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:33:22.290Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages cited in nearby Block-1 files and targeted pages: 15–36, 46–60, 61–80, 86–102; and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register behavior.
- Where the file documents instruction semantics such as TC/STD2, XCH, AD/SU overflow handling (PINC/MINC), NDX/EXTEND, MP/DV subinstruction sequencing, or SHINC/SHANC shift behavior, those semantics are corroborated by the cited PDFs and are marked "resolved (supported by AGCIS Issue 2/3)" below; remaining ambiguous items keep TODO:VERIFY.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
