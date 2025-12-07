# LINC — Load Increment (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.101–102, fig. 2‑43.

Summary
- Operation: Write into the addressed location the data provided by the Computer Test Set (used for manual loading of memory during testing). The address and data are supplied via the test-set keyboard.

Micro-op (C-like pseudocode)

```c
void LINC(uint16_t K, uint16_t data) {
    // Write the test-set supplied data into memory
    MEM[K] = data | (compute_parity_bit(data) << 15);

    // Advance normally
    Z = Z + 1;
    SQ = extract_order_code(MEM[Z] & 0x7FFF);
}
```

Notes
- In practice `LINC` reads `data` from the keyboard/test-set interface before writing.

Citations
- AGCIS Issue 2, pp.101–102, fig. 2‑43.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
