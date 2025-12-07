# STORE E — Store from GSE into E (Block-2)

Summary
- Peripheral store: take data supplied by GSE (via WAts) and store into E. Bank registers/addresses provided by GSE.

Detailed pseudocode

void STORE_E(uint16_t E_from_gse) {
    // Data is provided via WArs by GSE; write into E
    uint16_t value = read_wats();
    write_memory(E_from_gse, value);
}

Notes
- When E is in E-memory, editing rules apply during the write; GSE provides bank selection where required.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
