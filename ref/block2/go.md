# GO — Go (Block-2)

Summary
- GO is the error/restart instruction: when executed it transfers control to a fixed restart address (commonly 04000) to begin restart or recovery routines.

Detailed pseudocode

void GO(void) {
    STMIC_stage();

    // Fixed restart address (implementation-specific; AGC used 0o4000)
    const uint16_t restart_addr = 0o4000;

    // Place restart address into S and let STD2 finalize the transfer
    S = restart_addr;
    SQ = extract_order_code_for_restart();

    STD2_execute();
}

Notes
- GO is used by system error handlers; the exact restart target can be environment-dependent; the helper extract_order_code_for_restart() returns the proper order code for the restart vector.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
