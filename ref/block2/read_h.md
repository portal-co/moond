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