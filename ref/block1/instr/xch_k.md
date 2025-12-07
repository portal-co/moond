# XCH K — Exchange A with memory (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 20–22 (sections 2-19..2-21).

Summary
- Operation: Exchange the contents of accumulator `A` with the word at memory address `K`.
- Modernization: Presented as a single micro-op. Parity and sign bits preserved according to AGC conventions.

Micro-op (C-like pseudocode)

```c
void XCH_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC: fetch memory at K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    uint16_t mem = G & 0xFFFF;

    // Exchange A and memory
    uint16_t tmp = A;
    A = mem;
    MEM[S] = tmp;

    // Update parity and ordercode staging
    P = parity(A);
    B = mem & 0x7FFF;
    SQ = extract_order_code(B);

    // Advance PC
    Z = z + 1;
}
```

Citations
- AGCIS Issue 2, pp.20–22, §§2-19–2-21.

Notes
- The hardware performs this as multiple microcycles; we present the logical exchange suitable for emulation and documentation.

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.
