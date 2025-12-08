GO — C-like pseudocode

/* Start computer by executing instruction at fixed start address (02030). */
void GO(void) {
    address_t start = START_ADDRESS; // 02030 (octal)
    // Load start instruction into B and SQ just like TC but using start address
    B = MEM.read(start);
    SQ = ordercode_of(B);
    test_parity(B);
    // Set Z to start+1 per TC/GO semantics
    Z = start + 1;
    // Execute-first: SQG will now execute the order code in SQ
    SQG.execute_STD2();
}
