# WOR H — Write OR (A OR H) into A and H (Block-2)

Summary
- Compute bitwise OR of A and channel H, store the result in A and write it back to channel H.

Detailed pseudocode

void WOR_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    uint16_t res = A | ch;

    A = res;
    write_channel(H, res);

    B = I + 1;
    STD2_execute();
}

Notes
- write_channel must handle channel width and parity bits as per peripheral definitions.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
