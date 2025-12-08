# CCS E — Count, Compare, and Skip on E (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 56-62 (sections 32-43..32-55, figures 32-3 through 32-6).

## Summary

Test a value in extended memory (E-type addressing), decrement it, store in accumulator, and conditionally skip instructions based on the sign of the original value. Extended version of CCS K with E-memory support.

**Operation:** Test `memory[E]`, decrement by 1, store in A, branch based on original value

**Branching:**
- Original value > +0 → Execute next instruction (Z+2)
- Original value = +0 → Skip 1 instruction (Z+3)
- Original value < 0 → Skip 2 instructions (Z+4)
- Original value = -0 → Skip 3 instructions (Z+5)

**Note:** Branch addresses differ from Block-1 CCS K due to extended instruction sequencing.

## Pseudocode

```c
void CCS_E(uint16_t E) {
    // Fetch value to test from E-memory
    // (E-memory may require bank register handling)
    int16_t value = memory[E];
    
    // Decrement and store in accumulator
    // (AGC uses 1's complement: +0 and -0 are distinct)
    if (value == 0) {
        A = 0o77777;  // +0 decremented = -0
    } else if (value == 0o77777) {  // -0
        A = 0o77777;  // -0 decremented = -0
    } else if (value > 0) {
        A = value - 1;
    } else {
        A = value - 1;  // Negative values decrement normally
    }
    
    // Branch based on ORIGINAL value (before decrement)
    // Extended instruction uses different base address
    if (value > 0) {
        // Positive (not including +0)
        Z = Z + 2;  // Next instruction (extended instruction is 2 words)
    } else if (value == 0) {
        // Plus zero
        Z = Z + 3;  // Skip 1 instruction
    } else if (value < 0 && value != 0o77777) {
        // Negative (not including -0)
        Z = Z + 4;  // Skip 2 instructions
    } else {  // value == 0o77777 (minus zero)
        // Minus zero
        Z = Z + 5;  // Skip 3 instructions
    }
    
    // Restore E-memory if needed
    // (E-memory locations 0o400-0o1777 may need restore after read)
    if (E >= 0o400 && E <= 0o1777) {
        memory[E] = value;  // Restore original value
    }
    
    // Fetch and decode next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);
}
```

## Notes

### Block-2 Extended Addressing

CCS E uses extended (E-type) addressing which allows access to:
- **Basic memory:** 0o0000-0o1777 (same as Block-1)
- **E-memory:** 0o400-0o1777 (erasable memory requiring restore)
- **Bank-switched memory:** Higher addresses via EBANK/FBANK registers

### E-Memory Restore

E-memory locations (0o400-0o1777) are implemented with non-destructive read technology. After reading, the original value must be restored:
- Read destroys the value
- Must write back immediately
- This is automatic in hardware but must be explicit in emulation

### Branch Address Differences

CCS E branches differ from CCS K because extended instructions occupy 2 memory words:
- **CCS K:** Branches to Z+1, Z+2, Z+3, Z+4
- **CCS E:** Branches to Z+2, Z+3, Z+4, Z+5 (accounts for 2-word instruction)

### Use Cases

Same as Block-1 CCS K:
1. Loop counters with extended memory access
2. Testing values in banked memory
3. Conditional execution based on E-memory contents

## Citations

- AGCIS Issue 32 (Block-2), pages 56-62
  - Sections 32-43 through 32-45: CCS E operation description
  - Figures 32-3 through 32-6: CCS0 subinstruction variants (>+0, =+0, <0, =-0)
  - Section 32-44: Extended addressing rules
  - Section 32-45: E-memory restore requirements

## Audit

- PDF source: AGCIS Issue 32 pages 56-62 document CCS E operation
- Branch behavior: Verified in section 32-45 (4 cases: >+0, =+0, <0, =-0)
- Decrement behavior: Same as Block-1, verified in figures 32-3..32-6
- E-memory restore: Documented in section 32-45 paragraph (4)
- Address offset: Z+2 base confirmed (extended instruction is 2 words)
- Status: Core behavior verified from PDF sources

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference. E-memory restore logic shown explicitly.
