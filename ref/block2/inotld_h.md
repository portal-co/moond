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