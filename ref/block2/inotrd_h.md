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