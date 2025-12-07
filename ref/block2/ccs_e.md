# CCS E — Count, Compare, Skip on E (Block-2)

Summary
- Reads c(E), compares and branches depending on sign/zero; stores tested quantity into A and sets branch flip-flops.

Modernized pseudocode

void CCS_E(uint16_t E) {
    // Standard memory inquiry for E
    STMIC_stage(); // fetch B,S etc.

    uint16_t valE = read_memory(E);

    // Enter tested value into A (with appropriate complement rules per spec)
    A = valE;

    // Set B to I+1 (return/next) and set branch flip-flops based on valE
    B = I + 1;
    set_branch_flags(A);

    // Depending on sign/zero, advance Z by 0..3 and restore as required
    Z += branch_increment_for(A);

    // STD2 finalization
    STD2_execute();
}

Notes
- Subinstructions (CCS0 variants) are inlined into the function; restore/edit rules for E vs CP/F memory are preserved in `read_memory` and `restore_memory` helpers.

Detailed pseudocode

void CCS_E(uint16_t E) {
    // Standard memory inquiry (fetch B/S/X/Y and G as required)
    STMIC_stage();

    // Read E (handles E-memory edit/restore rules)
    uint16_t valE = read_memory(E);

    // Place tested value into A and set return bookkeeping
    A = valE;
    B = I + 1;

    // Set branch flip-flops based on A (BR1/BR2 logic)
    set_branch_flags(A);

    // Determine Z increment according to AGC rules:
    if (is_positive_nonzero(A)) {
        // no Z advance
        Z += 0;
    } else if (is_plus_zero(A)) {
        // plus-zero -> advance by 1
        Z += 1;
    } else if (is_negative_nonzero(A)) {
        // negative non-zero -> advance by 3
        Z += 3;
    } else { // minus-zero
        Z += 4;
    }

    // Finalize and call forward
    STD2_execute();
}

Helpers
- read_memory(E): performs memory inquiry with E-memory editing/restoration as specified by AGC IS; returns a 15-bit value (with sign bit as bit 16 when applicable).
- set_branch_flags(A): sets BR1/BR2 according to sign/zero and other rules (mirrors TSGN/TPZG tests).
- is_plus_zero/is_minus_zero helpers interpret the AGC-specific plus/minus-zero encodings.

Block-2 differences (placeholder)
- Keep this as a placeholder for any Block-2-specific branch rules discovered later.

Inline notes
- CCS_E in Block-2 is inlined to show CCS0/STD2 fusion: the STMIC, read, branch-flag setting, and Z increment are presented as a single atomic sequence to mirror the PDF's subinstruction grouping.
- Reference canonical helper: ref/Instruction.md::fetch_instruction_via_S and ref/STD2.md for STD2 semantics.

Edge cases / TODOs
- Sign encoding details (plus-zero vs minus-zero) are complex in AGC; where ambiguous, entries are marked with `TODO:VERIFY` for later validation against memos or hardware tests.
- Behavior for E-memory editing during CCS (restore/write-back) is marked `TODO:VERIFY` where the PDF's OCR is unclear.
