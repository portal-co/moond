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