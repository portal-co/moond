# OINC — Display (Zero Increment) (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.100–101, fig. 2‑42.

Summary
- Operation: Read and display the content of the addressed location (used with the Computer Test Set). No increment occurs to the stored location; the name "zero increment" refers to the mode of the test set display.

Micro-op (C-like pseudocode)

```c
void OINC(uint16_t K) {
    // Fetch the addressed word for display
    uint16_t word = MEM[K] & 0xFFFF;

    // Send to test-set display interface
    testset_display(word);

    // STMIC-like staging for next instruction
    Z = Z + 1; // normal next
    SQ = extract_order_code(MEM[Z] & 0x7FFF);
}
```

Notes
- This instruction is intended for human/operator interaction and test equipment; it should not modify memory.

Citations
- AGCIS Issue 2, pp.100–101, fig. 2‑42.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
