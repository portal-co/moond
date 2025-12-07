# RXOR H — Read XOR Channel H with A (Block-2)

Summary
- Compute bitwise exclusive-OR of register A and channel H and store the result in A.

Detailed pseudocode

void RXOR_H(uint16_t H) {
    STMIC_stage();

    uint16_t ch = read_channel(H);
    A = A ^ ch; // exclusive OR

    B = I + 1;
    STD2_execute();
}

Notes
- When used with SCALER/short channels, read_channel must normalize width; the XOR operates on the canonical 15-bit content used throughout the docs.