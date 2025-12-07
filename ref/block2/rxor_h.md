# RXOR H — Read XOR Channel H with A (Block-2)

Summary
- Compute bitwise exclusive-OR of register A and channel H and store the result in A.

Detailed pseudocode

void RXOR_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    A = A ^ ch; // exclusive OR

    B = I + 1;
    STD2_execute();
}

Notes
- When used with SCALER/short channels, read_channel must normalize width; the XOR operates on the canonical 15-bit content used throughout the docs.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.
