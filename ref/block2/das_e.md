# DAS E — Double Add to Storage E (Block-2)

Summary
- Add the double-precision quantity in A,L to the double-precision quantity at memory locations E and E+1.
- Store result in E and E+1, encode overflow into A.

Pseudocode

```c
// DAS E: Double-precision add to storage (Block-2)
// See ref/definitions/STD2.md for canonical subinstruction patterns
void DAS_E(uint16_t E) {
    // Read double-precision operands
    int32_t a_hi = sign_extend15(A);
    int32_t a_lo = sign_extend15(L);
    int32_t e_lo = sign_extend15(memory[E]);
    int32_t e_hi = sign_extend15(memory[E + 1]);

    // Compose 30-bit signed values
    int64_t a_full = ((int64_t)a_hi << 15) | (a_lo & 0x7FFF);
    int64_t e_full = ((int64_t)e_hi << 15) | (e_lo & 0x7FFF);

    // Perform double-precision addition
    int64_t sum = a_full + e_full;

    // Store result in E and E+1 (without overflow bits)
    memory[E] = (uint16_t)(sum & 0x7FFF);
    memory[E + 1] = (uint16_t)((sum >> 15) & 0x7FFF);

    // Encode overflow into A (if any)
    A = encode_double_overflow(sum);

    // Standard instruction completion (STD2 inline)
    // See ref/definitions/STD2.md
    Z = Z + 1;                          // Increment program counter
    uint16_t next = memory[Z];          // Fetch next instruction
    SQ = extract_order_code(next);      // Decode operation
}
```


## Semantics

```agc-sem
set tmp mem
set mem oc_add(L,tmp)
set mem_hi dp_add_hi(A,L,mem_hi,tmp)
set A 0
```

Notes
- encode_double_add_overflow(sum) implements AGC's rule for placing overflow indicators into A when double-precision storage overflows occur; helpers must preserve AGC bit semantics and editing for E-memory writes.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:33:22.290Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages cited in nearby Block-1 files and targeted pages: 15–36, 46–60, 61–80, 86–102; and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11 for register behavior.
- Where the file documents instruction semantics such as TC/STD2, XCH, AD/SU overflow handling (PINC/MINC), NDX/EXTEND, MP/DV subinstruction sequencing, or SHINC/SHANC shift behavior, those semantics are corroborated by the cited PDFs and are marked "resolved (supported by AGCIS Issue 2/3)" below; remaining ambiguous items keep TODO:VERIFY.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:01.141Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102) and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11) for CPU/register/adder/parity transfer rules. AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) was consulted where PGNS/scaler or word-format details apply.
- Status: instruction and register-transfer behaviors supported by those pages are marked resolved; edge-case timing and hardware-level evidence remain TODO:VERIFY and are tracked in ref/TODO_AUDIT.md.
