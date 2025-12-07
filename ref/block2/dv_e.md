# DV E — Divide by E (Block-2)

Summary
- Divide double-precision quantity (A,L) by single-precision divisor in E; writes quotient in A and remainder in L. Complex multi-subinstruction sequence is presented as one routine.

Pseudocode

void DV_E(uint16_t E) {
    // Prepare registers and memory access
    STMIC_stage();

    // Compose signed 30-bit dividend from A (high) and L (low)
    int32_t dividend = (sign_extend15(A) << 15) | (sign_extend15(L) & 0x7FFF);

    // Read divisor from E (15-bit signed); read_memory handles E-edit/restore
    int32_t divisor = sign_extend15(read_memory(E));

    // Division-by-zero handling follows AGCIS (may invoke RUPT or defined state changes)
    if (divisor == 0) {
        handle_divide_by_zero();
        return;
    }

    // Logical division using two's-complement arithmetic consistent with AGC method
    // The AGC implements DV via an iterative subinstruction sequence (DVO..DV6);
    // here we present the logical result while preserving signs and remainder semantics.
    int32_t quotient = dividend / divisor;
    int32_t remainder = dividend % divisor;

    // Store quotient (A) and remainder (L) as 15-bit quantities (helpers enforce AGC bit rules)
    A = (uint16_t)(quotient & 0x7FFF);
    L = (uint16_t)(remainder & 0x7FFF);

    // Update sign/overflow indicators according to AGC rules
    set_div_sign_and_overflow(quotient, remainder);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- This routine represents the logical effect of the DV0..DV7/DV4 sequence; detailed per-action timing and specific bit-level end-around-carry behavior are encapsulated in helpers (set_div_sign_and_overflow, handle_divide_by_zero) for clarity.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
