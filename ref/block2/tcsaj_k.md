# TCSAJ K — Transfer Control to Specified Address K (Peripheral) (Block-2)

Summary
- Peripheral-supplied transfer control: GSE supplies address K; TCSAJ K loads that address and branches to it.
- Used in ground test and peripheral operations.

Pseudocode

```c
// TCSAJ K: Transfer control with GSE-supplied address (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
void TCSAJ_K(uint16_t K_from_gse) {
    // Branch to GSE-supplied address
    Z = K_from_gse;
    uint16_t target_instr = memory[Z];
    SQ = extract_order_code(target_instr);
}
```

Notes
- For interactive GSE testing, FETCH/STORE interactions can be modeled by fetch_instruction_via_S and related helpers.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:33:47.148Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; corroborating instruction flow (STD2), NDX/EXTEND, PINC/MINC, SHINC/SHANC, MP/DV sequences, and register transfer rules.
- Where specific behavior (shift-and-add semantics, overflow counter operations, end-around carry prevention, UPRUPT signaling) is described in this file, it is supported by the cited PDFs and may be considered resolved; remaining nuanced timing/edge-case items retain TODO:VERIFY pending hardware memos.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
