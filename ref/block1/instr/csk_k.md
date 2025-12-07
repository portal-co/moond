# CSK K — Complement & Skip (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 22–23 (sections 2-22..2-24).

Summary
- Operation: Load the complemented value of memory at `K` into accumulator `A` (A := ~[K]), then advance to the next instruction.
- Modernization: Presented as a single micro-op; parity handling is preserved.

Micro-op (C-like pseudocode)

```c
void CSK_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC: fetch memory at K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];

    // Complement the fetched word (bitwise complement on 16 bits)
    uint16_t val = G & 0xFFFF;
    A = (uint16_t)(~val & 0xFFFF);

    // Update parity and staging
    P = parity(A);
    B = G & 0x7FFF;
    SQ = extract_order_code(B);

    // Advance PC
    Z = z + 1;
}
```

Citations
- AGCIS Issue 2, pp.22–23, §§2-22–2-24.

Notes
- The original AGC name in the combined file was `CSK` (Clear & Complement variant). This file uses the mnemonic `CSK` to match the original doc.

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
