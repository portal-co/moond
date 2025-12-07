MP K — C-like pseudocode (multiply A by memory K)

void MP_K(address_t K) {
    // High-level multiply: compute full product into (A, LP)
    word_t a = A_no_overflow(); // clear OV bit per AGCIS
    word_t k = MEM.read(K);

    product_t p = multiply_signed_14x14(a, k); // returns 28-bit product
    A = extract_high14(p);
    LP = extract_low14(p);

    // Sign bits in bit positions 16/15 handled per AGCIS; parity checks on operands performed during STMIC

    execute_MP_subsequence(); // MP0..MP3 handled by SQG per AGCIS
}
