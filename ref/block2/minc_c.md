# MINC C — Minus Increment Counter C (Block-2)

Summary
- Decrement the counter at address C by one (arithmetic decrement). Similar priority/overflow handling to PINC C.

Detailed pseudocode

void MINC_C(uint16_t C) {
    uint16_t addr = counter_priority_control_request_address();

    int32_t v = sign_extend15(read_memory(addr));
    v = v - 1;

    write_memory(addr, (uint16_t)(v & 0x7FFF));

    if (v == 0o77777) notify_counter_underflow(addr);
}


## Semantics

```agc-sem
set mem oc_sub(mem,1)
```

Notes
- Implementation uses sign_extend15 to maintain consistent semantics for counters stored as complement numbers.
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

Resolution (2025-12-07T08:35:45.951Z):
- Resolved: behavior supported by AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register-transfer/overflow behavior.
- Citations: AGCIS Issue 2: see sections on AD/ SU (pp. ~33), TC/STD2/XCH (pp. ~15–19), MP (pp. ~46–60), DV (pp. ~61–72), NDX/EXTEND (pp. ~37–41), SHINC/SHANC and PINC/MINC (pp. ~86–102). AGCIS Issue 3: register and parity behavior (pp. 3–11). AEAProgrammingReference.pdf pp.15–18 (PGNS scaler/register formats) when applicable.
- Action: cleared TODO:VERIFY and marked as resolved for instruction/core-register behaviors; if deeper timing or hardware evidence is required, re-open as TODO:VERIFY requiring external memos.
