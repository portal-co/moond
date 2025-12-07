# Adder — Operation and carry semantics (modernized)

Source: `agcis_3_central_processor.pdf` — pp.22–27, fig. 3-8, table 3-4.

Summary
- The Adder uses registers `X` and `Y` and per-bit carry logic. It supports end-around carry (carry-around from MSB back to LSB) as a normal operation and also allows "forced" carry input via control pulse `CI` (CBTI) for special arithmetic (pp.22–26).
- The Adder is organized into 16 identical sections (one per bit stick). The per-bit truth table for sum and carry is given in table 3‑4.

Key behaviors
- Normal addition: for each bit position, sum bit `u` and carry-out `c` follow the truth table (table 3-4). Each `c` becomes `ci` for next higher bit.
- End-around carry (CBTI): a carry out of bit 16 may be fed into bit 1; this is used for adding angular quantities and other wrap-around arithmetic.
- Forced carry / sign-handling: `CI` pulse can force a carry into the LSB (useful when adding 000001 to 177777 to propagate carries around).
- Sign bit switching: for certain address ranges (angular data addition) the sign bit written to WA16 is swapped with the overflow bit to implement sign-correction rules (pp.26–27).

Pseudocode primitives

```c
// Add two 16-bit signed values with optional forced carry-around semantics
int16_t agc_add(int16_t a, int16_t b, bool force_carry_around) {
    uint32_t sum = (uint16_t)a + (uint16_t)b;

    if (force_carry_around) {
        // Emulate CBTI: if carry out of bit 16 set, fold into LSB
        if (sum & 0x10000U) {
            sum = ((sum & 0xFFFFU) + 1) & 0xFFFFU; // end-around add
        }
    }

    // Emulate 16-bit two's complement wrap
    int16_t result = (int16_t)(sum & 0xFFFFU);

    // Compute overflow detection consistent with AGC rules
    bool overflow = ((a > 0 && b > 0 && result < 0) || (a < 0 && b < 0 && result >= 0));

    // When overflow and certain addresses apply, AGC writes overflow-bit into WA16 instead of sign-bit.
    if (overflow && is_angular_addressing()) {
        uint16_t to_wa16 = (~((uint16_t)result) >> 15) & 1; // overflow bit
        write_to_WA16(to_wa16);
    }

    return result;
}
```

Helper notes
- `is_angular_addressing()` models the address-range gating described in the doc (addresses 41 and 47–56 cause special sign behavior).
- `write_to_WA16()` models the write amplifier path that routes bit16 to WA16.

Citations
- AGCIS Issue 3, pp.22–27; table 3‑4 (truth table for bit section), figs. 3‑8..3‑10 (carry and sign handling).