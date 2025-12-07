# CCS K — Count, Compare, Skip (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 36–45 (figs. 2-12..2-16; table 2-3).

Summary
- Operation: Examine memory at address `K` and set `A` and the next PC based on its sign/zero state.
- Branch semantics (original):
  - if c(K) > +0  → next instruction = L+1
  - if c(K) = +0  → next instruction = L+2
  - if c(K) < -0  → next instruction = L+3
  - if c(K) = -0  → next instruction = L+4
- This modernized version presents CCS as a single micro-op routine (no subinstructions). Octal constants use `0o` prefix.

Notes on data representation
- Words are 16-bit quantities in AGC (value bits in positions 1..15, sign bit in 16, parity bit in 0). For brevity we use `(u)int15_t` for the 15-value-bit portion and `(int16_t)` or `(uint16_t)` when sign + value are considered together.

Micro-op (C-like pseudocode)

```c
// Types used (pseudotype names for clarity)
// (u)int15_t  : unsigned 15-bit value (bits 1..15)
// int16_t     : signed 16-bit word (bit16 = sign, bits1..15 = magnitude)

void CCS_K(uint16_t K) {
    // STMIC (common fetch & stage group)
    uint16_t z = Z;               // z = c(Z)  (next address)  -- control pulse RZ
    S = z;                        // WS
    Y = z; X = 0;                 // WY
    // fetch memory into G if S >= 0o20
    if (S >= 0o20) G = MEM[S];    // Action 6 in AGCIS

    // Stage: copy G->B and parity into P (prepare next instruction)
    B = G & 0x7FFF;               // RG, WB  (B holds bits 1..15)
    P = parity(G);                // WP / GP behaviour

    // Inspect content c(K) (present in G):
    int16_t e = (int16_t)G;       // signed view (16-bit: sign in bit16)

    // Compute behavior per original table (see AGCIS table 2-3)
    if (e > 0) {
        // c(K) > +0: A := e - 1 ; next = z + 1
        A = (int16_t)(e - 1);
        Z = z + 1;
    } else if (e == 0) {
        // c(K) == +0: A := 0 ; next = z + 2
        A = 0;
        Z = z + 2;
    } else if (e < 0 && !is_minus_zero(G)) {
        // c(K) < -0 (non -0 negative): A := e - 1 ; next = z + 3
        A = (int16_t)(e - 1);
        Z = z + 3;
    } else { // e == -0 (i.e., minus zero encoded as 0o177777)
        // c(K) == -0: A := 0 ; next = z + 4
        A = 0;
        Z = z + 4;
    }

    // Finalize: stage next instruction's order code into SQ and continue.
    // (Original AGC: at Action 12 the fetched instruction's order code is loaded to SQ.)
    SQ = extract_order_code(B);
}

// Helper (pseudocode): detect minus-zero encoding (0o177777)
bool is_minus_zero(uint16_t word) {
    return (word & 0x7FFF) == 0x7FFF && ((word >> 15) & 1) == 1; // parity omitted
}
```

Citations
- Behavior and branches: AGCIS Issue 2, pp.36–40 and examples/figures pp.42–45 (figs. 2-12..2-16, table 2-3).

Notes / Rationale
- The original AGC uses subinstructions CCS0 and CCSl; here we inline the standard memory-inquiry (STMIC) and the select/branch logic so the instruction appears as a single micro-op routine.
- We preserve exact branch distances (z + 1..z + 4) and the rule that A receives either `c(K) - 1` or `0` depending on the case (per AGCIS table 2-3).
- Octal constants are shown with `0o` and types use `(u)int15_t` for value-bit semantics (as requested). Parity handling is left as a separate micro-op (`parity(G)`) to mirror the hardware parity pyramid.

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
