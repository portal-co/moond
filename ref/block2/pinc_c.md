# PINC C — Plus Increment Counter C (Block-2)

Summary
- Increment counter at address C by one (wraps in TWO's complement or per counter semantics); special counter addresses signal priority control on overflow.

Detailed pseudocode

void PINC_C(uint16_t C) {
    // No typical STD2 memory sequencing is required beyond reading the counter address from Counter Priority Control
    uint16_t addr = counter_priority_control_request_address();

    int32_t v = sign_extend15(read_memory(addr));
    v = v + 1;

    write_memory(addr, (uint16_t)(v & 0x7FFF));

    // If overflow occurred, notify counter priority controller
    if (v == 0) notify_counter_overflow(addr);

    // PINC is involuntary; do not modify SQ. Still perform final housekeeping as AGC does
    // (bookkeeping left minimal for doc clarity)
}

Notes
- counter_priority_control_request_address() supplies the address selected by the Counter Priority Control hardware.