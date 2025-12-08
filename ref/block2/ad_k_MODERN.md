# AD K — Add (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 92-93 (section 32-128, figure 32-19).

## Summary

Add the value from memory address K to the accumulator. On overflow or underflow, schedule PINC/MINC counter operations.

**Operation:** `A = A + memory[K]`

**Overflow handling:** Positive overflow triggers PINC, negative underflow triggers MINC

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
    A = sum & 0o77777;  // Mask to 15 bits
    
    // Check for overflow (1's complement limits)
    if (sum > 0o37777) {        // Positive overflow
        schedule_PINC();         // Increment overflow counter
    } else if (sum < -0o40000) { // Negative underflow
        schedule_MINC();         // Decrement overflow counter
    }
    
    // Branch to next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    Z = Z + 1;                      // Increment program counter
    uint16_t next = memory[Z];      // Fetch next instruction
    SQ = extract_order_code(next);  // Decode operation
}
```

## Notes

### Block-2 vs Block-1

AD K behavior is identical in Block-1 and Block-2. The instruction uses basic (K-type) addressing without extended features.

### Overflow Detection

1's complement arithmetic with 15-bit magnitude:
- **Max positive:** `0o37777` (16383 decimal)
- **Max negative:** `-0o40000` (-16384 decimal)

Overflow sets flags for counter increment/decrement operations.

## Citations

- AGCIS Issue 32 (Block-2), pages 92-93
  - Section 32-128: AD K instruction description
  - Figure 32-19: Subinstruction AD0 timing diagram
- Overflow behavior: Section 32-128, paragraph describing PINC/MINC scheduling

## Audit

- PDF source: AGCIS Issue 32 pages 92-93 document AD K operation
- Overflow handling: PINC/MINC described in section 32-128
- Block-1 comparison: Behavior identical to Block-1 AD K
- Status: Core behavior verified from PDF source

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). STD2 inlined with reference.
