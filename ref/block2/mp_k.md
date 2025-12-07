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