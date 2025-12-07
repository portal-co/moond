# DV K — Divide (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 61–80 (figs. 2-26..2-35; table 2-5).

Summary
- Operation: Divide the A:Q pair by c(K), producing quotient in Q and remainder in A (modernized view). The AGC performed a restoring division via a sequence of shifts and conditional subtracts; we express the algorithm in concise C-like pseudocode.

Micro-op (C-like pseudocode)

```c
void DV_K(uint16_t K) {
    uint16_t z = Z;
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    int16_t divisor = (int16_t)(G & 0x7FFF);

    // Combine A:Q into a 32-bit signed numerator
    int32_t numerator = ((int32_t)(int16_t)A << 16) | (uint16_t)Q;
    int32_t quotient = 0;

    if (divisor == 0) {
        // Divide by zero behavior: original AGC did specific hardware behavior.
        // Here we document: set overflow or trap; for emulation, set quotient=0 and leave numerator unchanged.
        set_divide_by_zero_flag();
    } else {
        for (int i = 0; i < 16; ++i) {
            numerator <<= 1;
            quotient <<= 1;
            int32_t trial = numerator - ((int32_t)divisor << 16);
            if (trial >= 0) {
                numerator = trial;
                quotient |= 1;
            }
        }
    }

    // Place results back into A and Q
    A = (uint16_t)((numerator >> 16) & 0xFFFF);
    Q = (uint16_t)(quotient & 0xFFFF);

    Z = z + 1;
    SQ = extract_order_code(G & 0x7FFF);
}
```

Notes
- The algorithm above is restoring division emulation in high-level code. The real AGC implemented this over many cycles with dedicated microcode and shift hardware; this pseudocode is suitable for emulation and documentation.
- Divisor sign handling, rounding, and exact behavior for divide-by-zero are annotated to allow the emulator implementer to choose faithful action; cite original AGCIS pages for specifics.

Citations
- AGCIS Issue 2, pp.61–80 (figs. 2-26..2-35; table 2-5).
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
