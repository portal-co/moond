# DV K — Divide (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 63–79 (figs. 2-26..2-33, tables 2-5).

Summary
- Operation: Divide accumulator `A` by memory at `K`. Result: quotient in `A` (and/or `B/C` per final stage), remainder in `Q` (absolute remainder value in `Q`), and sign/LP set according to AGC rules.
- DVK uses `Q` (unique among Regular Instructions) to hold the remainder during the operation; subinstructions are `DV0` (setup) and repeated `DVl` steps (14 iterations), then `STD2` to finish.

Representation notes
- We use `(int16_t)` for signed 16-bit AGC words and `(u)int15_t` for raw 15-bit fields. Division algorithm follows AGC's complement-and-test approach (approach 3 in the doc): cycle and add, testing the high-order remainder bit each iteration.

Micro-op (C-like pseudocode)

```c
// High-level divide routine implementing DV K behaviour (pseudocode)
void DV_K(uint16_t K) {
    // STMIC: stage & fetch
    uint16_t z = Z;            // RZ
    S = z;                     // WS
    Y = z; X = 0;              // WY
    if (S >= 0o20) G = MEM[S]; // Action 6

    // Stage instruction code
    B = G & 0x7FFF;
    P = parity(G);

    // Load dividend, divisor
    int16_t a = A;             // dividend
    int16_t e = (int16_t)G;    // divisor

    // Handle sign and use 'complement cycling' approach (AGC approach #3)
    int sign_q_positive = ((a >= 0) == (e >= 0)); // quotient sign
    uint32_t abs_a = (uint32_t)(a < 0 ? -a : a) & 0x7FFF;
    uint32_t abs_e = (uint32_t)(e < 0 ? -e : e) & 0x7FFF;

    // Place abs_a into Q (complemented form as AGC does) for cycling
    uint32_t Q_reg = abs_a; // Q will be cycled and store remainder
    uint32_t B_reg = 0;     // B holds quotient bits shifted in complemented form

    // Pre-load LP and flags per DV0 (setup): LP and B initialize per sign logic
    LP = (sign_q_positive ? 0x0001 << 14 : 0xFFFF); // pseudo representation of LP setup

    // Main DV loop: 14 iterations (DVl repeated)
    for (int iter = 0; iter < 14; ++iter) {
        // Cycle Q left one (add zero at low end)
        Q_reg = (Q_reg << 1) & 0x3FFF;  // keep to high 14 bits logic

        // Add divisor to cycled remainder: u = (100000 V q) + e (per doc)
        uint32_t u = (0x10000 | Q_reg) + abs_e; // conceptually

        // If highest test bit of u is 0 => remainder positive => quotient bit = 1
        bool remainder_positive = ((u & 0x10000) == 0);
        if (remainder_positive) {
            // accept u as new remainder and shift B to include 1
            Q_reg = u & 0x3FFF;
            B_reg = (B_reg << 1) | 1;
        } else {
            // leave Q_reg as is (cycled) and shift B to include 0
            B_reg = (B_reg << 1) | 0;
        }

        // Update LP/B per AGC bit cycling behaviour (simplified mapping)
        // (actual hardware cycles B/LP and uses bit-16 tests; we capture logical effect)
    }

    // Finalize quotient and remainder semantics
    if (sign_q_positive) {
        A = (int16_t)(B_reg & 0x7FFF);  // quotient in complemented form per AGC final step
    } else {
        A = (int16_t)(-((int32_t)B_reg & 0x7FFF));
    }

    // Remainder placed in Q
    Q = (uint16_t)(Q_reg & 0x7FFF);

    // Advance PC
    Z = z + 1;
    SQ = extract_order_code(B);
}
```

Caveats and mapping
- The AGC hardware uses a specialized cycle-and-add with complement handling so that tests of the highest-order bit of the temporary `u` indicate correct subtractions. The pseudocode above maps that logic into a loop that cycles a `Q_reg` and forms `u` each iteration.
- The AGC uses `DV0` to initialize registers (A/Q/LP/B) and then executes `DVl` 14 times; we model the equivalent in a single routine.

Citations
- AGCIS Issue 2, pp.63–79 (figs. 2-26..2-33; tables 2-5) — details of DV0, DVl, and the iterative divide procedure.

Notes
- This is an algorithmic modernization intended to capture AGC semantics in readable C-like pseudocode. Where AGC uses precise control-pulse sequences (e.g., special cycling to/from `G`, `LP`, `B`, and `Q`), we preserve the logical effects (tested bit, quotient bit generation, remainder storage) rather than reproducing hardware-level pulses.
