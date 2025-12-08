SU K — C-like pseudocode (Subtract from K with EXTEND)

/* SU K is AD K but with complemented K (i.e., subtract). Requires EXTEND prior to set order code. */
void SU_K(address_t K) {
    word_t a = A;
    word_t k = MEM.read(K);
    word_t k_comp = complement_word(k);
    sum_t result = add_with_flags(a, k_comp);
    A = result.value;
    if (result.overflow) signal_OVCTR_increment();
    if (result.underflow) signal_OVCTR_decrement();
    SQG.execute_STD2();
}
