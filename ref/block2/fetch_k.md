# FETCH K — Fetch and display K (Block-2)

Summary
- Peripheral fetch: display content of K on GSE/WAts and restore prior bank registers; used by GSE to inspect memory/registers.

Detailed pseudocode

void FETCH_K(uint16_t K_from_gse) {
    // Fetch K and place content into WArs for GSE display
    Instruction inst = fetch_instruction_via_address(K_from_gse);

    // Display helpers write into WAts; not changing program flow
    display_on_wats(inst.raw_word);

    // Restore any modified bank registers and return
}

Notes
- This instruction is driven by external GSE and does not advance program sequencing in the usual way; implementation should mimic the AGC test harness behavior.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
