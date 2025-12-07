# MP K — Multiply (modernized, EXTEND preceding)

Source: `agcis_2_machine_instructions.pdf` — pages 53–63 (figs. 2-18..2-25, tables 2-4).

Summary
- Operation: Multiply accumulator `A` by memory at `K`. The high-order product is held in `A`, the low-order product in `LP` (14 value bits in each: bits 1..14 used for partial products, sign bits in 16/15 as per AGC convention).
- The AGC algorithm uses a sequence of micro-operations (MP0, MPl repeated, MP3) driven by a multiply counter (MPCTR). Here we present a single micro-op routine that performs the multiplication by executing the sequence of micro-ops described.

Representation notes
- We use `(int16_t)` to represent signed 16-bit words and `(u)int15_t` for the 15-value-bit field when needed. We keep the `LP` low-half product as a 16-bit register for simplicity, but only bits 1..14 are value bits as in AGC.

Micro-op (C-like pseudocode)

```c
// Pseudocode skeleton for MP K (high-level micro-op sequence)
void MP_K(uint16_t K) {
    // STMIC: fetch multiplier at K into G/B/P as usual
    uint16_t z = Z;             // RZ
    S = z;                      // WS
    Y = z; X = 0;               // WY
    if (S >= 0o20) G = MEM[S];  // Action 6

    // B holds instruction code (prepared by prior instruction); stage parity
    B = G & 0x7FFF;
    P = parity(G);

    // Read multiplier and multiplicand
    int16_t a = A;              // multiplicand (signed)
    int16_t e = (int16_t)G;     // multiplier (signed)

    // Initialize product registers
    int32_t product = 0;        // 32-bit accumulator for product calculation

    // Convert to absolute values with sign tracking (AGC uses sign bits in 16/15)
    int sign = ((a < 0) ^ (e < 0)) ? -1 : 1;
    uint32_t abs_a = (uint32_t)(a < 0 ? -a : a) & 0x7FFF;
    uint32_t abs_e = (uint32_t)(e < 0 ? -e : e) & 0x7FFF;

    // Multiply via shift-and-add (AGC methods produce partial-subtotals and shifts)
    for (int i = 0; i < 14; ++i) {
        if ((abs_e >> i) & 1) {
            product += ((uint32_t)abs_a) << i; // partial add
        }
    }

    // Apply sign
    if (sign < 0) product = -product;

    // Place high order into A, low order into LP according to AGC bit layout
    // Product has up to 28 bits (14+14); pick high 14 into A's value bits and low 14 into LP
    uint32_t low14  = product & 0x3FFF;            // low 14 bits
    uint32_t high14 = (product >> 14) & 0x3FFF;    // next 14 bits

    // Compose AGC-style 16-bit register words (value bits in 1..14, sign bits occupy 16/15)
    A = (int16_t)((sign < 0 ? 0x8000 : 0x0000) | (high14 & 0x3FFF));
    LP = (uint16_t)((sign < 0 ? 0x8000 : 0x0000) | (low14 & 0x3FFF));

    // Restore K if needed, advance PC
    Z = z + 1;
    SQ = extract_order_code(B);
}
```

Caveats and mapping to AGCIS
- The original AGC executes a sequence: `NDX`, `MP0` (setup), `MPl` (repeat 6 times), `MP3` (finish). The pseudocode above collapses those subinstructions into a single, explicit multiply implementation that yields the same A/LP contents.
- The AGC's micro-operations carefully handle sign propagation, bit cycling between `A` and `LP`, and partial-subtotal accumulation; the code here reproduces that logically via a shift-and-add loop with explicit sign handling.

Citations
- AGCIS Issue 2, pp.53–63 (figs. 2-18..2-25; table 2-4) describe the multiply principle, the subinstruction sequence, and register contents at the ends of the substeps.

Notes
- This file intentionally expresses multiplication as a clear C-style algorithm so it is easy to follow and to re-target to other platforms; it preserves the AGC semantics (A: high product, LP: low product) and parity/flag behavior is left to the micro-op layer (parity tests and WOVI inhibit rules would be emitted as extra micro-ops where required).
