# RAND H — Read-and-AND Channel H with A (Block-2)

Summary
- Perform bitwise AND between register A and channel H and store the logical product in A.

Detailed pseudocode

void RAND_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    A = A & ch; // logical AND (bits handled as 15/16 per channel semantics)

    B = I + 1;
    STD2_execute();
}

Notes
- Channel reads use read_channel(H) which returns the appropriate bit-width; the AND is performed on the lower 15 bits with special handling for parity/overflow bits where applicable.