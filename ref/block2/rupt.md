# RUPT — Interrupt Program Execution (Block-2)

Summary
- RUPT is the interrupt transfer instruction: saves current program state (B/Z) into reserved locations and transfers control to an interrupt routine as supplied by the Interrupt Priority Control.

Detailed pseudocode

void RUPT(void) {
    // STD-like memory inquiry for RUPT
    STMIC_stage();

    // Save current program return state into BRUPT and ZRUPT locations
    write_memory(0o17, B);       // BRUPT location (0017)
    write_memory(0o15, Z);       // ZRUPT location (0015)

    // Load transfer address provided by Interrupt Priority Control
    uint16_t target_addr = interrupt_priority_control_get_routine_address();

    // Place transfer into sequencing registers and call forward
    S = target_addr;
    SQ = extract_order_code_from_interrupt();

    // Finalize (STD2 handles loading B/S/SQ etc.)
    STD2_execute();
}

Notes
- interrupt_priority_control_get_routine_address() is an environment helper that responds to external interrupt logic; precise priority handling is outside this doc's scope.
- RUPT is typically preceded by INHINT/RELINT state handling.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.
