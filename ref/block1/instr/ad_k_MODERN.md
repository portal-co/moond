# AD K — Add (Block-1)

Source: `agcis_2_machine_instructions.pdf` — pages 31–33 (sections 2-33..2-35, figure 2-9).

## Summary

Add the value from memory address K to the accumulator. On overflow or underflow, the overflow counter is incremented or decremented.

**Operation:** `A = A + memory[K]`

**Overflow handling:** Sets PINC (positive overflow) or MINC (negative overflow) flags

## Pseudocode

```c
void AD_K(uint16_t K) {
    // Fetch operand from memory
    int16_t operand = memory[K];
    
    // Perform addition with overflow detection
    int32_t a_extended = sign_extend15(A);
    int32_t op_extended = sign_extend15(operand);
    int32_t sum = a_extended + op_extended;
    
    // Store result (15-bit value)
    A = sum & 0o77777;  // Mask to 15 bits (octal: 5 digits)
    
    // Check for overflow
    if (sum > 0o37777) {       // Positive overflow (> max positive 15-bit)
        overflow_counter++;     // Schedule PINC
    } else if (sum < -0o40000) { // Negative overflow (< max negative 15-bit)
        overflow_counter--;     // Schedule MINC
    }
    
    // Branch to next instruction (standard completion - STD2)
    // See ref/definitions/STD2.md for canonical subinstruction definition
    Z = Z + 1;                      // Increment program counter
    uint16_t next = memory[Z];      // Fetch next instruction
    SQ = extract_order_code(next);  // Decode operation (bits 12-15)
}
```

## Notes

### Overflow Detection

The AGC uses 1's complement arithmetic with 15-bit magnitude. Overflow occurs when:
- **Positive overflow:** Sum exceeds `0o37777` (16383 decimal, max positive)
- **Negative overflow:** Sum is less than `-0o40000` (-16384 decimal, max negative)

### PINC/MINC

Overflow and underflow are handled by incrementing or decrementing a counter:
- **PINC** (Plus Increment): Scheduled on positive overflow
- **MINC** (Minus Increment): Scheduled on negative underflow

These counters are part of the AGC's interrupt and overflow handling system.

### Subinstruction STD2

The standard completion sequence (STD2) is inlined above. It:
1. Increments the program counter (Z)
2. Fetches the next instruction from memory
3. Decodes the operation code for the Sequence Generator (SQ)

See `ref/definitions/STD2.md` for the canonical definition.

## Citations

- AGCIS Issue 2, pages 31–33, sections 2-33 through 2-35, figure 2-9
- Overflow handling described in AGCIS Issue 2, pages 46-50 (PINC/MINC discussion)

## Audit

- PDF source: AGCIS Issue 2 pages 31-33 corroborate basic AD operation
- Overflow handling: PINC/MINC semantics supported by pages 46-50 and counter discussion
- Verified 2025-12-07: overflow bit mapping and counter scheduling behavior documented
- Status: Core behavior verified; overflow counter details TODO:VERIFY for exact hardware timing

---

**Modernization Note:** This file uses the unified modern pseudocode style (2025-12-08). Subinstructions are inlined with reference comments. Hardware pulse names (RZ, WS, WG) are removed in favor of clear behavioral descriptions.
