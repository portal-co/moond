# EXTEND — Extra-code prefix instruction

Summary
- EXTEND is a Special Instruction that marks the following Basic or Extra-Code instruction as an "Extra-Code" variant by setting the EXT bit in SQ. It is normally used to enable order-code variants or to address larger F/extra fields; EXTEND itself executes STD2 as its concluding subinstruction.

Rules
- EXTEND must be executed immediately before an Extra-Code instruction that requires the EXT bit.
- EXTEND sets the EXT bit in SQ so that when the following STD2 executes the order-code is interpreted as an Extra-Code instruction.

Detailed pseudocode

void EXTEND_execute(void) {
    // Standard memory inquiry for EXTEND
    STMIC_stage();

    // Set the EXT bit in SQ so the next STD2 will interpret the next instruction as Extra-Code
    SQ = set_EXT_bit(SQ);

    // Finalize via STD2 (this will place the next instruction into B/S/SQ and call forward)
    STD2_execute();
}

Notes
- Implementation detail: set_EXT_bit(SQ) should set bit EXT (bit position used by AGC order-code extension) while preserving other SQ bits.
- EXTEND is rare in basic sequences; it is used to access extra-order-code variants and fixed-F behaviors documented in AGCIS.
- See per-instruction files for examples of use (e.g., NDX K, BZF F, BZMF F which rely on EXT/extra-code behavior).