# CA K — Clear and Add K (Block-2)

Summary
- Clears A and places content of memory location K into A (simple load).

Detailed pseudocode

void CA_K(uint16_t K) {
    // Standard memory inquiry for K
    STMIC_stage();

    // Read content from K (handles CP/E/F memory differences and E-memory restore/edit rules)
    A = read_memory(K);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- If K addresses E-memory the read/restore path obeys E-memory timing and editing rules; helper `read_memory` encapsulates those details.
- CA_K is the basic load into A; CA A/CA L/CA Q and CA ZERO variants follow the same pattern but target different registers.

Inline notes
- This Block-2 doc references ref/cpu/registers.md for canonical types (uint15_t/int15_t) and ref/definitions/Instruction.md for the Instruction type. In Block-2, small STMIC stages are typically inlined when timing is significant.

Edge cases / TODOs
- Memory bank selection when K targets F/E areas: TODO:VERIFY (PDF ambiguous).
- E-memory restore timing: TODO:VERIFY.
Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.
