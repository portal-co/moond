# CCS E — Count, Compare, Skip on E (Block-2)

Source: `agcis_32_block2_instructions.pdf` — pages 56–62 (AGCIS Issue 32).

Summary
- Reads c(E), compares and branches depending on sign/zero; stores tested quantity into A and sets branch flip-flops.
- Extended addressing variant supporting E-memory access.

Pseudocode

```c
// CCS E: Count, compare, and skip with extended addressing
// See ref/definitions/STD2.md for canonical subinstruction patterns
void CCS_E(uint16_t E) {
    // Save current program counter
    uint16_t next_addr = Z;
    
    // Fetch and test value from E-memory address
    // E-memory may require edit/restore operations
    int16_t test_value = (int16_t)memory[E];
    
    // Store tested value into accumulator
    A = test_value;
    
    // Conditional branch based on tested value
    // Four cases: positive, plus-zero, negative, minus-zero
    if (test_value > 0) {
        // c(E) > +0: skip 0 instructions
        Z = next_addr + 1;
    } else if (test_value == 0) {
        // c(E) == +0: skip 1 instruction
        Z = next_addr + 2;
    } else if (test_value < 0 && !is_minus_zero(test_value)) {
        // c(E) < 0 (not minus-zero): skip 2 instructions
        Z = next_addr + 3;
    } else {
        // c(E) == -0 (minus-zero): skip 3 instructions
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

Notes
- Unlike CCS K, CCS E supports extended addressing to access E-memory banks.
- E-memory access may involve edit/restore operations handled by memory subsystem.
- Tested value stored in A (different from CCS K which stores value-1).

Block-2 differences (placeholder)
- Keep this as a placeholder for any Block-2-specific branch rules discovered later.

Inline notes
- CCS_E in Block-2 is inlined to show CCS0/STD2 fusion: the STMIC, read, branch-flag setting, and Z increment are presented as a single atomic sequence to mirror the PDF's subinstruction grouping.
- Reference canonical helper: ref/Instruction.md::fetch_instruction_via_S and ref/STD2.md for STD2 semantics.

Edge cases / TODOs
- Sign encoding details (plus-zero vs minus-zero) are complex in AGC; where ambiguous, entries are marked with `TODO:VERIFY` for later validation against memos or hardware tests.
- Behavior for E-memory editing during CCS (restore/write-back) is marked `TODO:VERIFY` where the PDF's OCR is unclear.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
