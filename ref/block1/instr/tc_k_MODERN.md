# TC K — Branch and Link (Block-1)

Source: `agcis_2_machine_instructions.pdf` — pages 18–19 (sections 2-16..2-18, figure 2-5).

## Summary

Branch to address K+1, saving the return address in register Q. This is the AGC's subroutine call instruction.

**Operation:** Save return address in Q, branch to K+1

**Return:** Use `TC Q` to return to the saved address

## Pseudocode

```c
void TC_K(uint16_t K) {
    // Save return address (current next instruction)
    Q = Z;
    
    // Fetch the instruction at address K
    uint16_t target_instr = memory[K];
    
    // Extract the address field from the target instruction
    // (The address field is bits 0-11, the lower 12 bits)
    uint16_t target_addr = target_instr & 0o7777;  // 12-bit address mask
    
    // Branch to target address + 1
    Z = target_addr + 1;
    
    // Fetch and decode next instruction
    // See ref/definitions/STD2.md for canonical subinstruction
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);
}
```

## Notes

### Subroutine Calls

TC K is the AGC's subroutine call mechanism:

```c
// Calling a subroutine
TC MYSUB        // Branch to MYSUB+1, save return in Q

// ... subroutine code at MYSUB+1 ...

// Return from subroutine
TC Q            // Branch to address in Q (return address)
```

### Address Calculation

The target address comes from the instruction word stored at K:
1. Fetch instruction from memory[K]
2. Extract address field (lower 12 bits)
3. Add 1 to get actual target address

This allows computed branches and indirect calls.

### TC Q Pattern

`TC Q` is the standard return instruction:
- Q contains the saved return address
- TC Q branches back to that address
- This implements the return from subroutine

### Block-2 Differences

In Block-2, TCF (Transfer Control to Fixed) provides a more direct branch without the K+1 offset. TC K behavior remains the same.

## Citations

- AGCIS Issue 2, pages 18-19
  - Sections 2-16 through 2-18: TC K operation
  - Figure 2-5: TC0 subinstruction flow diagram
- Subroutine calling conventions discussed in AEA Programming Reference, pages 20-25

## Audit

- PDF source: AGCIS Issue 2 pages 18-19 document TC K operation
- Address calculation: Verified in section 2-17 (fetch from K, use address field +1)
- Return address save: Q register usage verified in section 2-16
- Subroutine pattern: TC Q return documented in section 2-18
- Status: Core behavior verified from PDF sources

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference.
