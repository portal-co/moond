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

// Inline notes: TC_K in Block-2 inlines the minimal STMIC behavior to preserve call-forward timing.
// Inlinee reference: ref/STD2.md (STD2_execute) and ref/Instruction.md (Instruction typedef)

void TC_K(uint16_t K) {
    // Inline STMIC: stage and prepare fetch
    S = Z; Y = Z; X = 0; // staging micro-ops
    if (S >= 0o20) {
        // inline memory read (handles CP/E/F distinctions)
        uint16_t tmp = read_memory(S);
    }

    // Save return address and set target
    Q = I + 1;
    S = K;
    SQ = extract_order_code(B); // EXT handling: TODO:VERIFY when EXT bit must be set

    // Finalize with STD2
    STD2_execute();
}

Inline notes
- TC_K in Block-2 presents the STMIC stages inline to show precise micro-op grouping; callers should reference ref/STD2.md for the finalization semantics.

Edge cases
- EXT bit handling for TC_K: TODO:VERIFY exact timing when EXT must be set (PDF ambiguous).
