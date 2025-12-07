# WAND H — Write AND (A AND H) into A and H (Block-2)

Summary
- Compute logical AND of A and channel H, store the result in A and also write it back to channel H.

Detailed pseudocode

void WAND_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    uint16_t res = A & ch;

    // Store logical product into both A and channel H
    A = res;
    write_channel(H, res);

    B = I + 1;
    STD2_execute();
}

Notes
- This instruction both reads and writes the addressed channel; write_channel must respect channel formatting and parity rules.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
