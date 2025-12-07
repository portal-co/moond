NDX K — C-like pseudocode (index next instruction)

void NDX_K(address_t K) {
    // Compute new instruction: B = c(z) + c(K)
    word_t z_next = Z + 1; // z = L+1 in subinstruction
    word_t k = MEM.read(K);
    B = add_wrap(z_next, k);

    // If overflow in order-code, resulting order code may be Extra (EXTEND semantics)
    if (ordercode_overflow(B)) {
        // Derived Extra Code; SQG will fetch OCN from B at STD2
    }

    // Restore K when applicable per AGCIS
    maybe_restore_K(K);

    // After executing the instruction in B, sequence continues at L+2 per AGCIS
    execute_instruction_in_B();
}
