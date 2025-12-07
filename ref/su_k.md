# SU K — Subtract (modernized, EXTEND preceding)

Source: `agcis_2_machine_instructions.pdf` — pages 46–53 (fig. 2-17 and MP discussion).

Summary
- Operation: Subtract memory at `K` from the accumulator `A`. With `EXTEND` preceding, the instruction code is an Extra Code Instruction (order code created by `NDX 5777`).
- Semantics (modernized): A := A - [K] (arithmetic). On overflow/underflow, adjust the overflow counter `OVCTR` (PINC / MINC), then continue.

Notes on representation
- We treat `A` and `K` as signed 16-bit quantities in pseudocode `(int16_t)` and use `(u)int15_t` to reference raw 15-bit value fields when needed.

Micro-op (C-like pseudocode)

```c
void SU_K(uint16_t K) {
    // STMIC (common stage) - fetch & stage
    uint16_t z = Z;            // RZ
    S = z;                     // WS
    Y = z; X = 0;              // WY
    if (S >= 0o20) G = MEM[S]; // read

    // Stage/prepare next instruction word into B/P
    B = G & 0x7FFF;
    P = parity(G);

    // Subtraction is implemented as addition of complemented [K]
    int16_t a_before = A;
    int16_t k_val = (int16_t)G;                   // c(K)

    // Complement K (two's-complement semantics as AGC used complemented add)
    int16_t km = ~k_val & 0xFFFF;                 // pseudo: complement

    // Sum and detect overflow/underflow
    int32_t sum = (int32_t)a_before + (int32_t)km + 1; // A + (~K) + 1 == A - K
    int16_t result = (int16_t)(sum & 0xFFFF);

    A = result;

    // Overflow / underflow handling
    if (sum > 0x7FFF) {
        // overflow
        schedule_PINC(); // increment OVCTR (PINC scheduled via control pulse WOVC)
    } else if (sum < -0x8000) {
        // underflow
        schedule_MINC(); // decrement OVCTR
    }

    // Restore K per addressing rules (if K in F/E memory) and continue
    Z = z + 1;  // STD behavior (advance PC)
    SQ = extract_order_code(B);
}
```

Citations
- AGCIS Issue 2, pp.46–53 (paragraphs 2-50..2-53 and figure 2-17). The document describes SU as identical in timing/behavior to `AD K` except using the complemented K; overflow/underflow are handled by `PINC`/`MINC`.

Notes
- The original machine performs subtraction by adding the complemented value of `K`. We express that explicitly via complement + 1 to form two's-complement subtraction.
- The pseudocode uses `schedule_PINC()` / `schedule_MINC()` to reflect that the AGC signals the Counter Priority Control which later initiates the PINC/MINC instruction (hardware-delayed increment/decrement of `OVCTR`).
