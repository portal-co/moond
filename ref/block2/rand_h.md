# RAND H — Read-and-AND Channel H with A (Block-2)

Summary
- Perform bitwise AND between register A and channel H and store the logical product in A.

Detailed pseudocode

void RAND_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    A = A & ch; // logical AND (bits handled as 15/16 per channel semantics)

    B = I + 1;
    STD2_execute();
}

Notes
- Channel reads use read_channel(H) which returns the appropriate bit-width; the AND is performed on the lower 15 bits with special handling for parity/overflow bits where applicable.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
