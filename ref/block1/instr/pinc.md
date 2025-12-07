# PINC — Increment Addressed Counter (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.90–92, figs. 2‑37.

Summary
- Operation: Increment by one the counter whose address is supplied by the Counter Increment Priority Control. Handle overflow by signaling the priority control and performing any required sign correction or follow-up action.
- Modernization: Modeled as an atomic counter increment routine that preserves AGC overflow behavior and parity checks.

Micro-op (C-like pseudocode)

```c
void PINC(void) {
    // Address of the counter to increment is provided by priority logic
    uint16_t ctr_addr = counter_priority_address();

    // Read counter value
    uint16_t e = MEM[ctr_addr] & 0x7FFF;

    // Increment with 16-bit wrap semantics
    uint32_t sum = (uint32_t)e + 1U;
    uint16_t new_val = (uint16_t)(sum & 0x7FFF);

    // Write back and parity
    MEM[ctr_addr] = new_val | (compute_parity_bit(new_val) << 15);

    // If overflow (sum >= 0x8000), notify the Counter Priority Control
    if (sum & 0x8000) notify_counter_overflow(ctr_addr);

    // Clear the Counter Priority Control for this event
    clear_counter_priority_request();
}
```

Notes
- The real AGC microcode uses WOVR/WG/RU pulses to write back and update parity; `compute_parity_bit()` represents that hardware parity generation.
- `notify_counter_overflow()` models the original behavior that sends a signal to the priority inputs.

Citations
- AGCIS Issue 2, pp.90–92, fig. 2‑37.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
