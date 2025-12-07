TC K — C-like pseudocode

/* Transfer control to K (TC K)
   - Execute instruction at K next; set Z=(TC K)+1, set Q=z (old next), restore K when appropriate.
   - Preserve overflow bit per AGCIS; inhibit interrupts if overflow bit would be lost.
*/
void TC_K(address_t K) {
    // Save return address
    Q = Z + 0;          // c(Q) = z

    // Compute new next address
    Z = B + 1;          // c(Z) = b(B) + 1 (TC K specifics)

    // Fetch instruction at K into G/B/P (STMIC)
    word_t f = MEM.read(K);
    // test parity and set alarm if needed
    test_parity(f);

    // Prepare SQ to execute instruction f
    SQ = opcode_of(f);

    // STD2-like sequencing performed by SQG
    execute_STD2();
}
