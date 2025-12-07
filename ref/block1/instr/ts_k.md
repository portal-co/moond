# TS K / TSK / TSO — Transfer to memory / Skip on overflow (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 24–28 (sections 2-25..2-28, figs. 2-6..2-7).

Summary
- Operation: Store `A` into memory at `K`. If `A` has overflow/underflow state, set `A` to a sentinel (+1 or -1 in AGC encoding) and skip the next instruction (advance `Z` by 2).
- Modernization: Presented as a single routine that tests overflow state, writes when appropriate, and advances `Z` accordingly.

Micro-op (C-like pseudocode)

```c
void TS_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC: fetch staging word; we use S to reference K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    B = G & 0x7FFF;

    // Test overflow/underflow flags on A
    if (!has_overflow(A) && !has_underflow(A)) {
        // Normal write
        MEM[S] = A;
        Z = z + 1;
    } else if (has_overflow(A)) {
        // Overflow: set A to +1 sentinel and skip next
        A = 0o1;
        Z = z + 2;
    } else {
        // Underflow: set A to -1 sentinel (AGC encoding) and skip
        A = 0o177776; // -1 in AGC representation for 16-bit
        Z = z + 2;
    }

    P = parity(A);
    SQ = extract_order_code(B);
}
```

Citations
- AGCIS Issue 2, pp.24–28, §§2-25–2-28 and figures referenced there.

Notes
- `TSK`/`TSO` variants represent control pulse differences; here we document the unified logical effect appropriate for emulation.

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
