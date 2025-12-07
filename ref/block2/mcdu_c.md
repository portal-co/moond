# MCDU C — Minus CDU C (Block-2)

Summary
- Decrement CDU-style counter at address C by one, cyclically.

Detailed pseudocode

void MCDU_C(uint16_t C) {
    uint16_t addr = counter_priority_control_request_address();
    uint16_t v = read_memory(addr);
    v = cyclic_twos_decrement(v);
    write_memory(addr, v);

    if (v == 0o77777) notify_counter_wrap(addr);
}

Notes
- Helper cyclic_twos_decrement implements TWO's-complement subtraction with proper wrapping semantics.