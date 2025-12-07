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

Detailed behavior
- Steps:
  1) STMIC_stage() to perform the standard memory inquiry for E;
  2) valE = read_memory(E) (handles E-memory editing/restore semantics);
  3) A = valE; B = I + 1; set_branch_flags(A) to update BR1/BR2 per sign/zero;
  4) compute Z increment (0..3) per AGC rules and advance Z accordingly;
  5) STD2_execute() finalizes and calls forward.
- Example: if c(E) is positive non-zero -> A=c(E), B=I+1, Z increment = 0; if plus-zero -> Z increment = 1; if negative non-zero -> Z += 3; if minus-zero -> Z += 4.

Subinstruction mapping
- CCS E expands CCS0 and STD2; CCS0 sets branch flip-flops and may cause Z to be advanced by 0..3 before STD2 finalization.

Block-2 differences (placeholder)
- Placeholder: record specific Block-2 deviations (if any) after targeted reading of Block-2 pages; do not re-read entire PDF unless requested.
