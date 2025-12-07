# CA K — Clear and Add K (Block-2)

Summary
- Clears A and places content of memory location K into A (simple load).

Pseudocode

void CA_K(uint16_t K) {
    STMIC_stage();
    A = read_memory(K);
    B = I + 1; // bookkeeping
    STD2_execute();
}

Notes
- If K addresses E-memory the read/restore path obeys E-memory timing and editing rules; helper `read_memory` encapsulates those details.