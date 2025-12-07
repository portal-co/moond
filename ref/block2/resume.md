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