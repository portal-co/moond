# STD2 — Standard finalizing subinstruction

Summary
- STD2 is the AGC standard concluding subinstruction executed to finalize most Basic and Extra-Code instructions. It increments the stage counter Z (normally by 1), prepares the next instruction fetch by loading registers B, S, and SQ as required, and triggers the call-forward mechanism that transfers control to the next instruction.

When STD2 runs
- STD2 is executed whenever the stage counter (ST) contains octal 2, or as the final subinstruction for many multi-subinstruction instructions. It performs the housekeeping that places the next instruction into B, S, and SQ and resets/advances registers for the following fetch.

Detailed pseudocode (helper used by per-instruction docs)

// Finalize current instruction and prepare next
void STD2_execute(void) {
    // 1) Fetch / load the next instruction word via register S. This abstracts:
    //    - If S addresses a CP register: read CP register into a temporary (handled by fetch_instruction_via_S)
    //    - If S addresses E memory: perform E read (with restore/edit semantics)
    //    - If S addresses F memory: perform F read
    Instruction next = fetch_instruction_via_S(S); // helper described in ref/Instruction.md

    // 2) Place fetched word into B, set S to next.address and SQ to the order code
    B = next.raw_word;
    S = next.address;
    SQ = next.order_code;

    // 3) Increment Z (stage/sequence register) as STD2's standard effect
    Z += 1;

    // 4) Perform any restores or writes required by prior E-memory operations
    perform_restores_if_needed();

    // 5) Trigger final control pulses (RB, WSQ, etc.) as required to make B/S/SQ visible to the SQG
    trigger_final_control_pulses();

    // After STD2 completes, the Sequence Generator will begin executing the subinstruction
    // determined by the new SQ/ST values.
}

Notes
- STD2_execute() is intentionally high-level: it captures the functional outcome used in per-instruction docs. Low-level timing and control-pulse ordering is preserved in the helpers referenced above for emulation fidelity.
- Helpers referenced:
  - fetch_instruction_via_S(addr): returns an Instruction parsed from the location/address described by S (handles CP/E/F distinctions and E-memory restore timing).
  - perform_restores_if_needed(): executes any E-memory or CP restores expected after reads.
  - trigger_final_control_pulses(): emits RB/WS/WSQ/RG etc. in the implementation.

See also: ref/Instruction.md for the Instruction type and ref/cpu/registers.md for bit layouts and order-code rules.