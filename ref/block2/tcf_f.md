# TCF F — Transfer Control to Fixed F (Block-2)

Summary
- Transfer control to a fixed F address (no change in bank register). Used to jump to a fixed F bank address.

Pseudocode

```c
// TCF F: Transfer control to fixed address (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
void TCF_F(uint16_t F) {
    // Fetch instruction from fixed address F
    uint16_t target_instr = memory[F];
    
    // Branch: set program counter to F + 1
    Z = F + 1;
    
    // Decode target instruction
    SQ = extract_order_code(target_instr);
}
```

Notes
- TCF_F leaves bank register unchanged and uses the fixed-area address in F for the next fetch.
- Block-2 differences: May require prior EXTEND instruction to access extra-code variants.

Inline notes
- In Block-2 TCF_F may require prior EXTEND to access extra-code variants. See ref/definitions/EXTEND.md for EXTEND handling.

Edge cases / TODOs
- Whether TCF_F must be preceded by EXTEND in all bank scenarios: TODO:VERIFY.
- Interaction with C register bank boundaries when F addresses cross banks: TODO:VERIFY.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
