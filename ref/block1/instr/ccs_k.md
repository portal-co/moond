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

Pseudocode

```c
// CCS K: Count, compare, and skip based on tested value
// See ref/definitions/STD2.md for canonical subinstruction patterns
void CCS_K(uint16_t K) {
    // Save current program counter
    uint16_t next_addr = Z;
    
    // Fetch and test value from memory address K
    int16_t test_value = (int16_t)memory[K];
    
    // Conditional branch based on tested value (see AGCIS table 2-3)
    // Four cases: positive, plus-zero, negative, minus-zero
    if (test_value > 0) {
        // c(K) > +0: A := test_value - 1, skip 0 instructions
        A = (int16_t)(test_value - 1);
        Z = next_addr + 1;
    } else if (test_value == 0) {
        // c(K) == +0: A := 0, skip 1 instruction
        A = 0;
        Z = next_addr + 2;
    } else if (test_value < 0 && !is_minus_zero(test_value)) {
        // c(K) < 0 (not minus-zero): A := test_value - 1, skip 2 instructions
        A = (int16_t)(test_value - 1);
        Z = next_addr + 3;
    } else {
        // c(K) == -0 (minus-zero = 0o177777): A := 0, skip 3 instructions
        A = 0;
        Z = next_addr + 4;
    }

    // Fetch and decode next instruction
    uint16_t next_instr = memory[Z];
    SQ = extract_order_code(next_instr);
}

// Helper: detect minus-zero encoding in AGC's ones-complement representation
bool is_minus_zero(int16_t value) {
    return (value & 0x7FFF) == 0x7FFF;  // All 15 value bits set = minus-zero
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
