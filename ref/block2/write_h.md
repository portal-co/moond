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
