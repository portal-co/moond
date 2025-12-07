# Write Amplifiers (WA01–WA16) — Behavior and gating (modernized)

Source: `agcis_3_central_processor.pdf` — pp.20–21, fig. 3-7.

Summary
- Sixteen Write Amplifiers (one per bit-stick) gate the outputs of registers and special buses into the G register and memory write path. Their inputs are many-source NOR-gate networks; any logical ONE at inputs produces the active `WL--` signal on the corresponding stick (pp.20–21).

Key points
- Each WA is an extended NOR with many inputs (gates A..H in the diagram). The WA produces `WL--` signals enabling writes into the destination flip-flops.
- WA outputs are routed to the G register and to memory when write cycles occur. Specific signals (`WG1G..WG6G`) select how WA outputs are placed into `G` (shift/cycle behavior — see `G` register docs) (pp.15–18).

Pseudocode helper

```c
// Gate a set of potential sources into the WA of a given stick
// src_mask is a bitmask of logical sources active; non-zero -> WA asserted
void evaluate_WA(int stick_idx, uint32_t src_mask) {
    bool active = (src_mask != 0);
    if (active) {
        set_write_line(stick_idx, true); // WL-- goes active (low-level in HW)
    } else {
        set_write_line(stick_idx, false);
    }
}

// During a write cycle: if WL active, write to destination flip-flops via G register
void perform_write_cycle(uint16_t address, uint16_t wa_values[16]) {
    // assemble word from WA lines
    uint16_t word = 0;
    for (int i = 0; i < 16; ++i) {
        word |= ((wa_values[i] & 1U) << i);
    }
    // write into memory or G depending on control
    if (is_memory_write_phase(address)) MEM[address] = word | (compute_parity_bit(word & 0x7FFF) << 15);
    else write_register("G", word);
}
```

Citations
- AGCIS Issue 3, pp.20–21 (fig. 3‑7, write amplifier block), and pp.15–18 (G register gating discussion).