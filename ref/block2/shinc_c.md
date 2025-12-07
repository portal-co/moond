# SHINC C — Shift Increment Counter C (Block-2)

Summary
- Shift counter content one place to the left and insert 0 into LSB (shift-left). If counter is flagged for shift-increment behavior shift-inserting a ZERO or ONE is supported per variant.

Detailed pseudocode

void SHINC_C(uint16_t C) {
    uint16_t addr = counter_priority_control_request_address();
    uint16_t v = read_memory(addr);

    // Shift one place left, insert zero in bit position 0
    v = (v << 1) & 0x7FFF;
    write_memory(addr, v);

    if (detect_overflow_on_shift(v)) signal_rupter_if_needed();
}

Notes
- Exact behavior depends on the counter configuration and whether a ONE should be shifted into LSB for SHANC; SHINC inserts ZERO.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
