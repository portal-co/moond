LINC — C-like pseudocode (Load Increment)

/* Load addressed location with data from keyboard/test set. Used for test set. */
void LINC(address_t K, word_t data) {
    // Write data into addressed location K
    word_t with_parity = set_parity_field(data);
    MEM.write(K, with_parity);
    // When used with test set, acknowledge via GSE discrete per AEA docs
}
