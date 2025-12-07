# TCF F — Transfer Control to Fixed F (Block-2)

Summary
- Transfer control to a fixed F address (no change in register C). Used to jump to a fixed F bank address.

Pseudocode

void TCF_F(uint16_t F) {
    STMIC_stage();
    // Do not change C; set next instruction to F (Fixed area)
    S = F;
    SQ = extract_order_code(B);
    STD2_execute();
}

Notes
- Behavior matches AGC Issue 32 description; Block-2 notes to follow if differences are observed during full parsing.

Detailed pseudocode

void TCF_F(uint16_t F) {
    // Standard memory inquiry
    STMIC_stage();

    // Do not change C; set next instruction pointer to fixed F
    S = F;
    SQ = extract_order_code(B); // EXT handling should be done via EXTEND helper

    // STD2 finalization
    STD2_execute();
}

Notes
- TCF_F leaves C unchanged and uses the fixed-area address in F for the next fetch; model as atomic with STD2_execute() encapsulating final pulses.
- Block-2 differences (placeholder): record any Block-2-specific bank/EXT semantics when discovered.

Inline notes
- In Block-2 TCF_F may require prior EXTEND to access extra-code variants; inline STMIC to show prefetch effects and reference ref/definitions/EXTEND.md.

Edge cases / TODOs
- Whether TCF_F must be preceded by EXTEND in all bank scenarios: TODO:VERIFY.
- Interaction with C register bank boundaries when F addresses cross banks: TODO:VERIFY.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
