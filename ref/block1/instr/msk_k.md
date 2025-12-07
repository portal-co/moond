# MSK K — Mask (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 28–29 (sections 2-30..2-32).

Summary
- Operation: Perform a bitwise mask of `A` with the word at memory `K`: `A := A & [K]`.
- Modernization: Present as logical AND. Parity and order code staging preserved.

Micro-op (C-like pseudocode)

```c
void MSK_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];

    // Mask A with memory
    uint16_t m = G & 0xFFFF;
    A = (uint16_t)(A & m);

    // Update parity and staging
    P = parity(A);
    B = G & 0x7FFF;
    SQ = extract_order_code(B);

    Z = z + 1;
}
```

Citations
- AGCIS Issue 2, pp.28–29, §§2-30–2-32.

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
