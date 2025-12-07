# RPT — Interrupt Program (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.86–87, figs. 2‑34..2‑35, table 2‑6.

Summary
- Operation: Save the current program context (Z and B) into reserved storage (ZRUPT/BRUPT), transfer control to an interrupt service routine determined by the Interrupt Priority Control, and inhibit further interrupts until resumed.
- Modernization: Present as a single atomic routine that encapsulates the original RPTl/RPT3/STD2 subinstructions.

Micro-op (C-like pseudocode)

```c
void RPT(void) {
    // 1. Save Z and B into preserved interrupt storage
    ZRUPT = Z;    // store next-address of interrupted code
    BRUPT = B;    // store B (next-order word) of interrupted code

    // 2. Obtain interrupt handler entry address from Priority Control
    uint16_t handler_addr = interrupt_priority_address(); // from hardware priority inputs

    // 3. Transfer control to the handler
    // Write the handler entry address into Z and initiate its execution
    Z = handler_addr;
    SQ = fetch_order_code_at(handler_addr);

    // 4. Inhibit further program interrupts until RSM executes
    set_interrupt_in_progress(true);

    // 5. Reset the interrupt request in the Priority Control
    clear_interrupt_request();
}
```

Notes
- The original AGC implementation uses microcoded subinstructions and specialized transfer routines (table 2‑6); this pseudocode captures the logical effect for emulation and documentation.
- `interrupt_priority_address()` and `fetch_order_code_at()` are helper primitives that model the priority control and memory-order-code fetch.

Citations
- AGCIS Issue 2, pp.86–87, figs. 2‑34..2‑35, table 2‑6.