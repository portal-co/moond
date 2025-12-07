# WOR H — Write OR (A OR H) into A and H (Block-2)

Summary
- Compute bitwise OR of A and channel H, store the result in A and write it back to channel H.

Detailed pseudocode

void WOR_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    uint16_t res = A | ch;

    A = res;
    write_channel(H, res);

    B = I + 1;
    STD2_execute();
}

Notes
- write_channel must handle channel width and parity bits as per peripheral definitions.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:33:47.148Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; corroborating instruction flow (STD2), NDX/EXTEND, PINC/MINC, SHINC/SHANC, MP/DV sequences, and register transfer rules.
- Where specific behavior (shift-and-add semantics, overflow counter operations, end-around carry prevention, UPRUPT signaling) is described in this file, it is supported by the cited PDFs and may be considered resolved; remaining nuanced timing/edge-case items retain TODO:VERIFY pending hardware memos.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
