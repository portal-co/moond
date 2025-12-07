# SU E — Subtract E from A (Block-2)

Summary
- Subtract content of memory location E from register A and store the difference in A (A = A - c(E)).

Detailed pseudocode

void SU_E(uint16_t E) {
    // Standard memory inquiry and read
    STMIC_stage();

    int32_t a = sign_extend15(A);
    int32_t e = sign_extend15(read_memory(E));

    int32_t diff = a - e;

    // Store result as 15-bit value and set overflow flags per AGC rules
    A = (uint16_t)(diff & 0x7FFF);
    set_sub_overflow_flags(diff);

    B = I + 1;
    STD2_execute();
}

Notes
- This collapses the SU0/STD2 subinstruction behavior; helpers maintain AGC-specific overflow and sign behavior.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
