TCSA — C-like pseudocode (Start at Specified Address)

/* Start at SA provided by test set. RSA supplies SA into S at Action 1. */
void TCSA(address_t SA) {
    // Similar to GO but SA is provided externally
    B = MEM.read(SA);
    SQ = ordercode_of(B);
    test_parity(B);
    Z = SA + 1;
    SQG.execute_STD2();
}
