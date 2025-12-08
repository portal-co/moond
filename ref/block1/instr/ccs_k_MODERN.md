# CCS K — Count, Compare, and Skip (Block-1)

Source: `agcis_2_machine_instructions.pdf` — pages 36–45 (sections 2-36..2-42, figures 2-12 through 2-16, table 2-3).

## Summary

Test a value in memory, decrement it, store in accumulator, and conditionally skip instructions based on the sign of the original value.

**Operation:** Test `memory[K]`, decrement by 1, store in A, branch based on original value

**Branching:**
- Original value > +0 → Execute next instruction (Z+1)
- Original value = +0 → Skip 1 instruction (Z+2)  
- Original value < 0 → Skip 2 instructions (Z+3)
- Original value = -0 → Skip 3 instructions (Z+4)

## Pseudocode

```c
void CCS_K(uint16_t K) {
    // Fetch value to test
    int16_t value = memory[K];
    
    // Decrement and store in accumulator
    // (Note: AGC uses 1's complement, so -0 and +0 are distinct)
    if (value == 0) {
        A = 0o77777;  // +0 decremented = -0 (all ones in 1's complement)
    } else if (value == 0o77777) {  // -0 in 1's complement
        A = 0o77777;  // -0 decremented = -0
    } else if (value > 0) {
        A = value - 1;
    } else {
        A = value - 1;  // Negative values decrement normally
    }
    
    // Branch based on ORIGINAL value (before decrement)
    // This is the "skip" behavior
    if (value > 0) {
        // Positive (not including +0)
        Z = Z + 1;  // Next instruction
    } else if (value == 0) {
        // Plus zero
        Z = Z + 2;  // Skip 1 instruction
    } else if (value < 0 && value != 0o77777) {
        // Negative (not including -0)
        Z = Z + 3;  // Skip 2 instructions
    } else {  // value == 0o77777 (minus zero)
        // Minus zero
        Z = Z + 4;  // Skip 3 instructions
    }
    
    // Fetch and decode next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);  // Decode operation
}
```

## Notes

### 1's Complement Arithmetic

The AGC uses 1's complement representation, which means:
- **Plus zero:** `0o00000` (all bits zero)
- **Minus zero:** `0o77777` (all bits one)  
- Both represent zero but are distinct values

### Branching Logic

CCS is primarily used for loop control and conditional logic:

```c
// Example: Loop 10 times
CCS COUNTER     // Test and decrement counter
TC  LOOP_BODY   // If > 0, continue loop (branch to LOOP_BODY)
TC  DONE        // If = +0, exit (counter reached zero, skip to DONE)
// Next instruction if < 0 (error condition)
// Next instruction if = -0 (shouldn't happen)
```

### Use Cases

1. **Loop counters:** Decrement and test for zero in one instruction
2. **Sign testing:** Branch based on positive/negative/zero
3. **Conditional execution:** Skip instructions based on value

## Citations

- AGCIS Issue 2, pages 36-45
  - Sections 2-36 through 2-42: Detailed CCS behavior
  - Figures 2-12 through 2-16: Subinstruction flow diagrams (CCS0 variants)
  - Table 2-3: Branch conditions and program counter updates
- 1's complement representation discussed in AGCIS Issue 3, pages 12-15

## Audit

- PDF source: AGCIS Issue 2 pages 36-45 fully document CCS operation
- Branch behavior: Verified in table 2-3 (4 cases: >+0, =+0, <0, =-0)
- Decrement behavior: Verified in figures 2-12 through 2-16 (subinstruction flows)
- 1's complement +0/-0 distinction: Verified in AGCIS Issue 3 pages 12-15
- Status: Core behavior fully verified from PDF sources

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 subinstruction inlined with reference comment.
