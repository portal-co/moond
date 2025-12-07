# GO — Computer GO (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.100–101 (section 2‑121..2‑123).

Summary
- Operation: Start the computer by loading and executing the instruction at a predefined start address (02030 in AGC examples). Equivalent in effect to `TC` but uses the start-read control pulse to fetch the start address.

Micro-op (C-like pseudocode)

```c
void GO(void) {
    // Start address is provided by hardware (start register)
    uint16_t start_addr = start_address_from_hardware(); // typically 0o2030

    // Stage & fetch the start instruction
    Z = start_addr;
    B = MEM[start_addr] & 0x7FFF;
    SQ = extract_order_code(B);
}
```

Notes
- GO is functionally equivalent to `TC` but uses the `RSTRT` pulse to obtain the start address. It leaves the machine ready to execute the start instruction.

Citations
- AGCIS Issue 2, pp.100–101.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.
