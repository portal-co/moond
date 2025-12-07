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
