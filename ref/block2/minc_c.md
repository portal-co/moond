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

Notes
- Implementation uses sign_extend15 to maintain consistent semantics for counters stored as complement numbers.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
