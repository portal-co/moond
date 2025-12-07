# TC K — Transfer Control to K (Block-2)

Summary
- Transfer control to location K (next instruction comes from K). Stores return address in Q and advances.

Behavior notes (Block-2)
- Block-2 semantics largely match Block-1 for TC K; any Block-2 divergences will be noted in the per-file differences area.

Pseudocode (modernized)

void TC_K(uint16_t K) {
    // Fetch instruction at K and schedule return
    // STMIC_stage() abstracts the standard memory inquiry (fetch B/S/X/Y and G)
    STMIC_stage(); // read B,S etc. as needed

    // Save return address (I+1) in Q and set next instruction pointer to K
    Q = I + 1;            // conceptual: store return address
    S = K;                // set sequence to fetch from K
    SQ = extract_order_code(B); // set order code from B into SQ

    // Call forward to the instruction at K (STD2 equivalent inlined)
    STD2_execute();
}

Notes
- `STD2_execute()` stands for the standard finalizing subinstruction that increments Z and prepares next fetch; this is inlined in the real AGC but represented here as a helper for clarity.