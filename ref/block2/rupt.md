# RUPT — Interrupt Program Execution (Block-2)

Summary
- RUPT is the interrupt transfer instruction: saves current program state (instruction and PC) into reserved locations and transfers control to interrupt routine.

Pseudocode

```c
// RUPT: Save state and transfer to interrupt handler (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
void RUPT(void) {
    // Save current program state to interrupt save locations
    memory[0o17] = memory[Z];    // Save current instruction to BRUPT
    memory[0o15] = Z;            // Save program counter to ZRUPT

    // Get interrupt vector from Interrupt Priority Control
    uint16_t interrupt_vector = get_interrupt_vector();

    // Branch to interrupt handler
    Z = interrupt_vector;
    uint16_t handler_instr = memory[Z];
    SQ = extract_order_code(handler_instr);
}
```

## Semantics

```agc-sem
set mem_at(0o17) deref(Z)
set mem_at(0o16) Z
branch mem_at(0o4)
```

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

Audit resolution (2025-12-07T08:33:47.148Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; corroborating instruction flow (STD2), NDX/EXTEND, PINC/MINC, SHINC/SHANC, MP/DV sequences, and register transfer rules.
- Where specific behavior (shift-and-add semantics, overflow counter operations, end-around carry prevention, UPRUPT signaling) is described in this file, it is supported by the cited PDFs and may be considered resolved; remaining nuanced timing/edge-case items retain TODO:VERIFY pending hardware memos.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
