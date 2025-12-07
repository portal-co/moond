# WRITE H — Write A to Channel H (Block-2)

Summary
- Write the content of register A into channel H. Channel address H is provided by the instruction or by GSE.

Detailed pseudocode

void WRITE_H(uint16_t H) {
    STMIC_stage();

    // Write A into channel H (write_channel handles width/parity/format)
    write_channel(H, A);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- write_channel must implement channel-specific formatting and parity as required by the peripheral; see ref/cpu/write_amplifiers.md for channel I/O notes.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
