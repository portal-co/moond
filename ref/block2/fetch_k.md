# FETCH K — Fetch and display K (Block-2)

Summary
- Peripheral fetch: display content of K on GSE/WAts and restore prior bank registers; used by GSE to inspect memory/registers.

Detailed pseudocode

void FETCH_K(uint16_t K_from_gse) {
    // Fetch K and place content into WArs for GSE display
    Instruction inst = fetch_instruction_via_address(K_from_gse);

    // Display helpers write into WAts; not changing program flow
    display_on_wats(inst.raw_word);

    // Restore any modified bank registers and return
}

Notes
- This instruction is driven by external GSE and does not advance program sequencing in the usual way; implementation should mimic the AGC test harness behavior.