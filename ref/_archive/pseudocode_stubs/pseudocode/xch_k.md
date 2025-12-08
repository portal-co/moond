XCH K — C-like pseudocode (flip-flop and memory variants)

/* Exchange accumulator with location K. For flip-flop registers, semantics differ. */
void XCH_K(address_t K) {
    word_t a = A;
    word_t k = MEM.read(K);
    test_parity(k);
    A = k;
    // When K is a flip-flop register (<0020) a write semantics apply per AGCIS
    MEM.write(K, a);
    adjust_overflow_sign_bits_on_exchange(a, k);
    SQG.execute_STD2();
}
