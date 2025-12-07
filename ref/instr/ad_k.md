# AD K — Add memory into A (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 31–33 (sections 2-33..2-35, figure 2-9).

Summary
- Operation: Perform arithmetic `A := A + [K]`. On overflow/underflow, schedule increment/decrement of the overflow counter (PINC/MINC semantics).
- Modernization: Single micro-op that performs the add and signals overflow actions via helper calls.

Micro-op (C-like pseudocode)

```c
void AD_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];

    int32_t sum = (int32_t)(int16_t)A + (int32_t)(int16_t)(G & 0xFFFF);
    A = (int16_t)(sum & 0xFFFF);

    if (sum > 0x7FFF) schedule_PINC();
    else if (sum < -0x8000) schedule_MINC();

    P = parity(A);
    B = G & 0x7FFF;
    SQ = extract_order_code(B);
    Z = z + 1;
}
```

Citations
- AGCIS Issue 2, pp.31–33, §§2-33–2-35 and figure 2-9.

Notes
- `schedule_PINC()` / `schedule_MINC()` are documented elsewhere (these emulate increment/decrement of the overflow-counter chain as in AGC hardware).
