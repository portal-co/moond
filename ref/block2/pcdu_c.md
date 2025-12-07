# PCDU C — Plus CDU C (Block-2)

Summary
- Increment CDU-style counter (cyclic TWO's-complement counters) at address C by one, wrapping in TWO's-complement arithmetic.

Detailed pseudocode

void PCDU_C(uint16_t C) {
    uint16_t addr = counter_priority_control_request_address();
    // Counters are cyclic TWO's complement; use helper that performs cyclic increment
    uint16_t v = read_memory(addr);
    v = cyclic_twos_increment(v);
    write_memory(addr, v);

    // If wrap-around occurred, notify as needed
    if (v == 0) notify_counter_wrap(addr);
}

Notes
- cyclic_twos_increment handles the TWO's-complement specifics; see ref/block2/msu_e.md for TWO's-complement helpers.