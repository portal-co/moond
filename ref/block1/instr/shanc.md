# SHANC — Shift Content and Add One (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.96–99, fig. 2‑41.

Summary
- Operation: Shift the addressed counter left by one position and add one to the result. Used in serial-to-parallel conversion for uplink/radar data.
- Modernization: Modeled as an atomic shift-and-add routine with the same interrupt and overflow handling as `SHINC`, but with an addition of one.

Micro-op (C-like pseudocode)

```c
void SHANC(void) {
    uint16_t ctr_addr = counter_priority_address();

    int32_t e = (int32_t)(int16_t)(MEM[ctr_addr] & 0xFFFF);

    // Shift left and add one
    int32_t result = (e << 1) + 1;

    // Overflow/underflow handling
    if (result > 0x7FFF || result < -0x8000) {
        result = reverse_sign_bit(result);
        notify_shift_overflow(ctr_addr);
    }

    uint16_t out = (uint16_t)(result & 0xFFFF);
    MEM[ctr_addr] = out | (compute_parity_bit(out) << 15);

    // If relevant flag bit was present, signal priority control
    if (flag_bit_was_set((int16_t)e)) trigger_uprupt();

    clear_counter_priority_request();
}
```

Citations
- AGCIS Issue 2, pp.96–99, fig. 2‑41.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
