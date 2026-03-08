# NDX E — Index Next Basic Instruction (Block-2)

Summary
- Indexing instruction: add content of location E to the next instruction address to derive the effective instruction.

Pseudocode

```c
// NDX E: Index next instruction (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
void NDX_E(uint16_t E) {
    // Fetch indexing offset from E
    int16_t index_value = sign_extend15(memory[E]);

    // Fetch next instruction
    uint16_t next_instr = memory[Z + 1];

    // Apply index to instruction address field
    // AGC indexing adds offset to address bits, preserving opcode
    uint16_t indexed_instr = apply_index(next_instr, index_value);

    // Store modified instruction for execution
    Z = Z + 1;
    SQ = extract_order_code(indexed_instr);
}
```

Helpers
- derive_instruction(next_inst, idx): performs bitwise addition of idx to the instruction word/address per AGC rules, preserves EXT semantics, and returns normalized Basic Instruction (not an Extra-Code instruction unless valid).


## Semantics

```agc-sem
set tmp mem
set deref(Z) oc_add(deref(Z),tmp)
```

Notes
- NDX E collapses NDXO/NDXI subinstructions into one logical operation for documentation; preserve precise bit/quarter-code arithmetic in derive_instruction for emulation fidelity.
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

Resolution (2025-12-07T08:35:45.951Z):
- Resolved: behavior supported by AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register-transfer/overflow behavior.
- Citations: AGCIS Issue 2: see sections on AD/ SU (pp. ~33), TC/STD2/XCH (pp. ~15–19), MP (pp. ~46–60), DV (pp. ~61–72), NDX/EXTEND (pp. ~37–41), SHINC/SHANC and PINC/MINC (pp. ~86–102). AGCIS Issue 3: register and parity behavior (pp. 3–11). AEAProgrammingReference.pdf pp.15–18 (PGNS scaler/register formats) when applicable.
- Action: cleared TODO:VERIFY and marked as resolved for instruction/core-register behaviors; if deeper timing or hardware evidence is required, re-open as TODO:VERIFY requiring external memos.
