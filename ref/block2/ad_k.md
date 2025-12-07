# AD K — Add K (Block-2)

Summary
- Add content of memory location K to register A; result in A, set overflow bits per AGC rules.

Detailed pseudocode

void AD_K(uint16_t K) {
    // Standard memory inquiry
    STMIC_stage();

    // Read K and perform signed 15-bit add
    int32_t a = sign_extend15(A);
    int32_t k = sign_extend15(read_memory(K));

    int32_t sum = a + k;

    // Store result (low 15 bits) into A and set overflow per AGC semantics
    A = (uint16_t)(sum & 0x7FFF);
    set_add_overflow_flags(sum);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- set_add_overflow_flags(sum) should implement AGC's overflow detection that sets sign/overflow flip-flops and encodes +1/-1 into A where AGC specifies (see ADS/DAS for related behavior)."
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
