# READ H — Read Channel H into A (Block-2)

Summary
- Read the content of channel H into register A. Channel addresses are supplied by GSE or by the instruction's operand, depending on usage.

Detailed pseudocode

void READ_H(uint16_t H) {
    // Standard memory/channel inquiry
    STMIC_stage();

    // Read channel H (read_channel handles I/O mapping and 14/16-bit channel widths)
    A = read_channel(H);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- read_channel(H) returns the channel content as a 15-bit/16-bit value; for display-only channels the helper should zero-extend or sign-extend as appropriate.
- Use ref/STD2.md and ref/Instruction.md for helpers and type conventions.

Inline notes
- Channel I/O docs reference ref/cpu/write_amplifiers.md and ref/cpu/registers.md for canonical types and width handling; Block-2 docs inline small channel staging where timing matters.

Edge cases / TODOs
- Channel width normalization for 14-bit SCALER channels: TODO:VERIFY exact alignment.
- Parity/formatting rules for certain legacy channels: TODO:VERIFY.
Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.
