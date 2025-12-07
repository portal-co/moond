# INOTRD H — In-Out Read H (Block-2)

Summary
- GSE-driven read of a channel: fetch channel H and display it on WAt(s). Channel address provided by GSE.

Detailed pseudocode

void INOTRD_H(uint16_t H_from_gse) {
    uint16_t ch = read_channel(H_from_gse);
    display_on_wats(ch);
}

Notes
- This is a peripheral test instruction used by GSE to inspect channels.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
