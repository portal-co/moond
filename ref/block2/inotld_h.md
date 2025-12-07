# INOTLD H — In-Out Load H (Block-2)

Summary
- GSE-driven write: load supplied WAt data into channel H. Channel address H supplied by GSE.

Detailed pseudocode

void INOTLD_H(uint16_t H_from_gse) {
    uint16_t v = read_wats();
    write_channel(H_from_gse, v);
}

Notes
- Used for console testing; write_channel performs channel-format encoding.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
