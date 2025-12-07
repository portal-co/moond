# RSM — Resume Program (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.89–91, fig. 2‑36.

Summary
- Operation: Restore the saved program context (ZRUPT/BRUPT) back into `Z` and `B`, release the interrupt-in-progress inhibition, and continue execution at the restored instruction.
- Modernization: Presented as a single routine (NDX0 + RSM in the original) that atomically restores and resumes.

Micro-op (C-like pseudocode)

```c
void RSM(void) {
    // Restore saved registers
    B = BRUPT;
    Z = ZRUPT;

    // Clear interrupt-in-progress so new interrupts may occur
    set_interrupt_in_progress(false);

    // Load the order code of the restored B and execute
    SQ = extract_order_code(B);
}
```

Notes
- The original RSM is executed via `NDX 0025` (special NDX) and a short microsequence; this routine captures the intended high-level semantics for emulator documentation.

Citations
- AGCIS Issue 2, pp.89–91, fig. 2‑36.