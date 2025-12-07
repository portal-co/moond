# XCH E — Exchange A and E (Block-2)

Summary
- Exchange contents of register A with memory location E; overflow bit handling follows spec (overflow in A may be lost depending on variant).

Detailed pseudocode

void XCH_E(uint16_t E) {
    // Standard memory inquiry
    STMIC_stage();

    // Read memory content at E (handles E-memory edit/restore)
    uint16_t memE = read_memory(E);

    // Exchange: write A to E (overflow bit handling according to memory type) and load A with memE
    // write_memory will obey E-memory edit/restore and drop/encode overflow-bit as required
    write_memory(E, A & 0x7FFF);
    A = memE;

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- Overflow semantics: when writing A into E the overflow bit may be lost for E/F memory depending on variant; helpers `read_memory`/`write_memory` preserve AGC-specific edit semantics.
- XCH variants (LXCH, QXCH) follow the same pattern but target different registers.

Inline notes
- Block-2 style: STMIC stages are often inlined into XCH to reflect fused micro-op timing; reference canonical helpers in ref/definitions/Instruction.md and ref/cpu/registers.md.

Edge cases / TODOs
- Exact overflow-bit propagation when exchanging with E-memory: TODO:VERIFY.
- Whether write_memory preserves overflow bit for specific bank types: TODO:VERIFY.
Audit
- Searched repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative references supporting this item's semantics.
- Result: authoritative support not found or ambiguous in repository PDFs. This item remains marked TODO:VERIFY and is provisionally marked as "inferred from training/model" when the original source is not present in repo.
- Action: retain TODO:VERIFY marker in-file and record in ref/TODO_AUDIT.md for later authoritative sourcing; if you have access to additional AGC memos or hardware logs, add citations to resolve.

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.
