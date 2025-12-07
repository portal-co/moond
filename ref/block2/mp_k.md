# MP K — Multiply by K (Block-2)

Summary
- Multiply A by memory K producing a double-precision product in A (high) and L (low). Uses multi-step subinstructions; modernized into one routine.

Pseudocode

// Inlined helper notes: STMIC_stage() is implemented inline here for Block-2 to reflect condensed subinstruction timing.
// Inline notes:
// - Inlining rationale: Block-2 original microcode fused STMIC and early MP subinstructions in short sequences; inlining preserves that visibility for timing-sensitive emulation.
// - Reference (canonical helper): ref/Instruction.md::fetch_instruction_via_S and ref/STD2.md for STD2 semantics.

void MP_K(uint16_t K) {
    // STMIC_stage() inlined: perform address staging and operand fetch as early micro-ops
    S = Z; Y = Z; X = 0;
    if (S >= 0o20) {
        // G fetch from memory
        uint16_t g = read_memory(S); // inline read (handles E/F/CP distinctions)
    }

    // Logical operation (same as Block-1 canonical version)
    int32_t multiplicand = sign_extend15(read_memory(K));
    int32_t multiplier   = sign_extend15(A);

    int32_t full_product = multiplicand * multiplier; // fits in signed 30 bits

    L = (uint16_t)(full_product & 0x7FFF);
    A = (uint16_t)((full_product >> 15) & 0x7FFF);

    set_product_sign_and_overflow(full_product); // TODO:VERIFY exact overflow encoding

    B = I + 1;
    STD2_execute();
}

Notes
- Inline notes above explain why early micro-ops are shown inline for Block-2.
- Edge cases: exact overflow encoding and timing of write-backs to E-memory are marked with `TODO:VERIFY` where the PDF is ambiguous.
Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

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
