# READ H — Read Channel H into A (Block-2)

Source: `agcis_32_block2_instructions.pdf` — pages 151–153 (AGCIS Issue 32).

Summary
- Read the content of I/O channel H into register A. Channel addresses map to hardware interfaces.

Pseudocode

```c
// READ H: Read I/O channel into accumulator
// See ref/definitions/STD2.md for canonical subinstruction patterns
void READ_H(uint16_t H) {
    // Read from I/O channel (handles 14/16-bit channel widths)
    // Channel mapping defined in AEA Programming Reference pages 15-18
    A = read_channel(H);

    // Standard instruction completion (STD2 inline)
    // See ref/definitions/STD2.md
    Z = Z + 1;                          // Increment program counter
    uint16_t next = memory[Z];          // Fetch next instruction
    SQ = extract_order_code(next);      // Decode operation
}
```

## Semantics

```agc-sem
set A chan
```

Notes
- read_channel(H) returns the channel content as a 15-bit/16-bit value; for display-only channels the helper should zero-extend or sign-extend as appropriate.
- Use ref/STD2.md and ref/Instruction.md for helpers and type conventions.

Inline notes
- Channel I/O docs reference ref/cpu/write_amplifiers.md and ref/cpu/registers.md for canonical types and width handling; Block-2 docs inline small channel staging where timing matters.

Edge cases / TODOs
- Channel width normalization for 14-bit SCALER channels: TODO:VERIFY exact alignment.
- Parity/formatting rules for certain legacy channels: TODO:VERIFY.
Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.

Audit resolution (2025-12-07T08:33:47.148Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; corroborating instruction flow (STD2), NDX/EXTEND, PINC/MINC, SHINC/SHANC, MP/DV sequences, and register transfer rules.
- Where specific behavior (shift-and-add semantics, overflow counter operations, end-around carry prevention, UPRUPT signaling) is described in this file, it is supported by the cited PDFs and may be considered resolved; remaining nuanced timing/edge-case items retain TODO:VERIFY pending hardware memos.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
