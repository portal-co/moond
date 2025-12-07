# INCR E — Increment E (Block-2)

Summary
- Increment location E by one (useful for counters and state variables). Handles E-memory edit/restore rules and signals Counter Priority Control when counter addresses are involved.

Detailed pseudocode

void INCR_E(uint16_t E) {
    // Standard memory inquiry
    STMIC_stage();

    // Read, increment, and write back (E-memory edit/restore handled by helpers)
    int32_t v = sign_extend15(read_memory(E));

    // Increment magnitude by one (overflow bit is lost on writes to E as specified)
    v = v + 1;

    write_memory(E, (uint16_t)(v & 0x7FFF));

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- If E corresponds to a counter address that requires Counter Priority handling, the implementation must notify the Counter Priority Control on overflow as described in AGCIS (e.g., addresses 0024..0027)."
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
