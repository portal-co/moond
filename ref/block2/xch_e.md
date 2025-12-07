# XCH E — Exchange A and E (Block-2)

Summary
- Exchange contents of register A with memory location E; overflow bit handling follows spec (overflow in A may be lost depending on variant).

Pseudocode

void XCH_E(uint16_t E) {
    STMIC_stage();

    uint16_t memE = read_memory(E);
    // Exchange (overflow handling: mem -> A, A -> mem may drop overflow bit as per spec)
    write_memory(E, A & 0x7FFF); // example: store 15-bit value to memory location (overflow dropped)
    A = memE; // A receives mem value

    B = I + 1;
    STD2_execute();
}

Notes
- Precise overflow-bit behavior is preserved by `read_memory`/`write_memory` helpers; Block-2 differences can be noted later if found.