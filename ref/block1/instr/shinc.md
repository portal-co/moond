# SHINC — Shift Content of Addressed Counter (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.92–95, figs. 2‑39..2‑40.

Summary
- Operation: Shift the addressed counter left by one position (multiply by two) with sign handling. Used for converting serial uplink/radar-range data into parallel words.
- Modernization: Single routine implementing the shift, sign/overflow handling, and program-interrupt signaling when a flagged bit is shifted into the test position.

Micro-op (C-like pseudocode)

```c
void SHINC(void) {
    uint16_t ctr_addr = counter_priority_address();

    // Read counter as signed 16-bit (15-value bits + sign)
    int16_t e = (int16_t)(MEM[ctr_addr] & 0xFFFF);

    // Shift left by one (logical on value bits, preserve sign rules per AGC)
    int32_t shifted = (int32_t)e << 1;

    // Handle overflow: reverse sign bit if required and prevent end-around carry
    if (shifted > 0x7FFF || shifted < -0x8000) {
        shifted = reverse_sign_bit(shifted);
        notify_shift_overflow(ctr_addr);
    }

    // Store back lower 16 bits with parity
    uint16_t out = (uint16_t)(shifted & 0xFFFF);
    MEM[ctr_addr] = out | (compute_parity_bit(out) << 15);

    // If bit 15 (flag) was set before the shift, signal program priority (UPRUPT)
    if (flag_bit_was_set(e)) trigger_uprupt();

    clear_counter_priority_request();
}
```

Notes
- `flag_bit_was_set()` models the check of bit position 15 described in the AGCIS text (used to indicate end-of-uplink-word).
- The hardware uses several micro-action pulses; this routine focuses on logical result and side-effects (UPRUPT signaling, overflow handling).

Citations
- AGCIS Issue 2, pp.92–96, figs. 2‑39..2‑40.