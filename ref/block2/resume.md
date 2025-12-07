# RESUME — Resume Interrupted Program (Block-2)

Summary
- RESUME restores saved BRUPT/ZRUPT information from reserved locations and resumes execution of the interrupted program if no higher-priority RUPT is pending.

Detailed pseudocode

void RESUME(void) {
    STMIC_stage();

    // Restore BRUPT and ZRUPT
    B = read_memory(0o17);    // BRUPT
    Z = read_memory(0o15);    // ZRUPT

    // Place the restored instruction into sequencing registers and call-forward
    Instruction inst = fetch_instruction_via_B(B);
    S = inst.address;
    SQ = inst.order_code;

    STD2_execute();
}

Notes
- RESUME relies on saved BRUPT/ZRUPT locations (0017/0015) and checks the INHINT/RELINT state prior to resuming; helper functions abstract these checks.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.
