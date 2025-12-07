# MINC — Decrement Addressed Counter (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.91–94, figs. 2‑38.

Summary
- Operation: Decrement by one the counter whose address is supplied by the Counter Increment Priority Control. Handle underflow by signaling the priority control and performing any required sign correction.
- Modernization: Modeled as an atomic counter decrement routine that preserves AGC underflow behavior and parity checks.

Micro-op (C-like pseudocode)

```c
void MINC(void) {
    // Address from priority logic
    uint16_t ctr_addr = counter_priority_address();

    // Read counter value (15-bit magnitude)
    uint16_t e = MEM[ctr_addr] & 0x7FFF;

    // Decrement with wrap-around semantics
    uint32_t diff = (uint32_t)((int32_t)e - 1);
    uint16_t new_val = (uint16_t)(diff & 0x7FFF);

    // Write back with parity
    MEM[ctr_addr] = new_val | (compute_parity_bit(new_val) << 15);

    // If underflow (borrow from bit15), notify counter priority logic
    if ((diff & 0x8000) != 0) notify_counter_underflow(ctr_addr);

    // Clear counter priority request
    clear_counter_priority_request();
}
```

Notes
- The AGC microcode reads `177776` into Y for certain decrements; we model the arithmetic result and signal underflow similarly.

Citations
- AGCIS Issue 2, pp.91–94, fig. 2‑38.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
