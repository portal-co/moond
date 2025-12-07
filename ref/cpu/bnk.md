# BNK — Bank Register (modernized)

Source: `agcis_3_central_processor.pdf` — pp.28–31, table 3-5.

Summary
- `BNK` selects which fixed-memory bank (0..21) is currently active for addressing; each bank contains 1024 locations. BNK has five flip-flops and responds to write/clear/read gating similar to other registers.

Behavior
- BNK outputs (RO..R4) are routed to bank-selector logic together with bits 11..12 of `S` to choose one of the rope gates (RPG1..RPG6) that select the bank pseudo-address mapping (pp.28–29).

Pseudocode primitives

```c
// Set bank register value (0..21)
void set_bank(uint8_t bank) {
    // clamp to valid range
    bank = bank % 22;
    uint16_t val = (uint16_t)(bank & 0x1F);
    write_register("BNK", val);
    update_bank_selector(val);
}

uint8_t current_bank(void) {
    return (uint8_t)(read_register("BNK") & 0x1F);
}

// Pseudo: map (bank, offset) -> physical memory address
uint32_t banked_address(uint8_t bank, uint16_t offset) {
    // each bank = 1024 locations => bank * 1024 + offset
    return (uint32_t)bank * 1024 + (offset & 0x03FF);
}
```

Citations
- AGCIS Issue 3, pp.28–31; Table 3‑5 (BNK register outputs and gating).