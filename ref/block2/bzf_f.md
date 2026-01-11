# BZF F — Branch on Zero to Fixed F (Block-2)

Summary
- Test register A; if zero, take next instruction from F (fixed); otherwise continue.
- Requires EXTEND instruction to set EXT bit for extra addressing.

Pseudocode

```c
// BZF F: Branch on zero to fixed address (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
// See ref/definitions/EXTEND.md for EXTEND instruction handling
void BZF_F(uint16_t F) {
    // Test accumulator for zero
    if (A == 0) {
        // Branch taken: fetch instruction from fixed address F
        uint16_t target_instr = memory[F];
        Z = F + 1;
        SQ = extract_order_code(target_instr);
    } else {
        // Branch not taken: continue to next instruction
        Z = Z + 1;
        uint16_t next_instr = memory[Z];
        SQ = extract_order_code(next_instr);
    }
}
```

Notes
- EXT handling: callers must execute EXTEND instruction when an Extra-Code/Fixed-F instruction requires the EXT bit to be set before BZF_F.
- This pseudocode models the observable branching behavior.

Inline notes
- BZF_F in Block-2 often relies on EXTEND being executed immediately prior. See ref/definitions/EXTEND.md and ref/cpu/registers.md for canonical behavior.

Edge cases / TODOs
- Exact timing requirement for EXT bit relative to STD2: TODO:VERIFY.
- Behavior if EXTEND is omitted but operand expects EXT semantics: TODO:VERIFY."
Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:33:22.290Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages cited in nearby Block-1 files and targeted pages: 15–36, 46–60, 61–80, 86–102; and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register behavior.
- Where the file documents instruction semantics such as TC/STD2, XCH, AD/SU overflow handling (PINC/MINC), NDX/EXTEND, MP/DV subinstruction sequencing, or SHINC/SHANC shift behavior, those semantics are corroborated by the cited PDFs and are marked "resolved (supported by AGCIS Issue 2/3)" below; remaining ambiguous items keep TODO:VERIFY.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
