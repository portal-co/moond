# TC K — Transfer Control (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 18–19 (sections 2-16..2-18).

Summary
- Operation: Save the current next-address (`Z`) into `Q` and set the program counter `Z` to `K + 1`, then begin execution at the order code fetched from `K`.
- Modernization: Presented as a single micro-op routine (no subinstructions). Octal constants use `0o` prefix.

Micro-op (C-like pseudocode)

```c
void TC_K(uint16_t K) {
    uint16_t z = Z;            // next address

    // STMIC: stage memory inquiry for K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    B = G & 0x7FFF;
    P = parity(G);

    // Save return address and set new PC
    Q = z;
    Z = (uint16_t)(B + 1);

    // Load next order code
    SQ = extract_order_code(B);
}
```

Citations
- AGCIS Issue 2, pp.18–19, §§2-16–2-18.

Notes
- The original AGC implements this with `TC0` + `STD2` subinstructions; we inline the behavior so the instruction appears atomic for emulator documentation.

Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.
