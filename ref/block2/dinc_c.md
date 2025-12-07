# DINC C — Diminish Increment Counter C (Block-2)

Summary
- DINC decreases the magnitude of the addressed counter by one (useful for counters that represent magnitudes). For positive numbers this subtracts one; for negative numbers this adds one (preserving sign).

Detailed pseudocode

void DINC_C(uint16_t C) {
    uint16_t addr = counter_priority_control_request_address();
    int32_t v = sign_extend15(read_memory(addr));

    if (v > 0) v = v - 1;
    else if (v < 0) v = v + 1;

    write_memory(addr, (uint16_t)(v & 0x7FFF));
}

Notes
- The Counter Priority Control is notified on special overflow conditions per hardware rules.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
