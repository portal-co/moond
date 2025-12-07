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