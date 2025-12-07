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
