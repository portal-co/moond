# CA K — Clear and Add K (Block-2)

Summary
- Clears A and places content of memory location K into A (simple load).

Detailed pseudocode

void CA_K(uint16_t K) {
    // Standard memory inquiry for K
    STMIC_stage();

    // Read content from K (handles CP/E/F memory differences and E-memory restore/edit rules)
    A = read_memory(K);

    // Bookkeeping and finalize
    B = I + 1;
    STD2_execute();
}

Notes
- If K addresses E-memory the read/restore path obeys E-memory timing and editing rules; helper `read_memory` encapsulates those details.
- CA_K is the basic load into A; CA A/CA L/CA Q and CA ZERO variants follow the same pattern but target different registers.