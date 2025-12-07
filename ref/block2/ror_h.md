# ROR H — Read OR Channel H with A (Block-2)

Summary
- Compute bitwise OR of register A and channel H and store the logical sum in A.

Detailed pseudocode

void ROR_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    A = A | ch; // logical OR

    B = I + 1;
    STD2_execute();
}

Notes
- For OR operations involving SCALER or other 14-bit channels, read_channel must provide the correct bit alignment; OR is performed on the canonical 15-bit field used in AGC docs.