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