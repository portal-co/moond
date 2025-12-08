# INCR E — Increment (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 138-140 (section 32-186).

## Summary

Increment the value at memory address E by one. Standard addition operation useful for counters and loop variables.

**Operation:** `memory[E] = memory[E] + 1`

**Use case:** Loop counters, state variables, incremental updates

## Pseudocode

```c
void INCR_E(uint16_t E) {
    // Fetch current value
    int16_t value = memory[E];
    
    // Increment by 1 (standard addition)
    int32_t result = value + 1;
    
    // Handle overflow in 1's complement
    if (result > 0o37777) {
        // Positive overflow: wrap to negative
        result = result - 0o100000;  // Wrap around in 1's complement
    }
    
    // Store result (15-bit value, overflow bit discarded)
    memory[E] = result & 0o77777;
    
    // Restore E-memory if needed
    if (E >= 0o400 && E <= 0o1777) {
        // E-memory restore handled by write
    }
    
    // Branch to next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    Z = Z + 1;
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);
}
```

## Notes

### INCR vs AUG

- **INCR E:** Standard addition (+1: -1 + 1 = 0)
- **AUG E:** Magnitude increase (+1 becomes +2, -1 becomes -2)

INCR performs arithmetic addition, AUG increases magnitude.

### Counter Operations

When E addresses a counter (0o24-0o27):
- Counter Priority Control may be triggered
- Overflow handling varies by counter type
- See AGCIS Issue 32 section on Counter Instructions

### 1's Complement Wrap

In 1's complement:
- **Max positive (0o37777)** + 1 → **Min negative (0o40000)**
- **-0 (0o77777)** + 1 → **+0 (0o00000)**
- **-1 (0o77776)** + 1 → **-0 (0o77777)**

### Use Cases

1. **Loop counters:** `INCR COUNTER` increments loop index
2. **State machines:** Advance state by one
3. **Timers:** Increment tick counter
4. **Sequences:** Step through indexed operations

### Related Instructions

- **DIM E:** Decrement (opposite of INCR)
- **AUG E:** Augment magnitude
- **ADS E:** Add to storage (adds accumulator to memory)

## Citations

- AGCIS Issue 32 (Block-2), pages 138-140
  - Section 32-186: INCR E instruction description
  - Note: INCR appears in context after ADS E section 32-171
- Counter operations: AGCIS Issue 32, sections 32-287..32-318

## Audit

- PDF source: AGCIS Issue 32 pages 138-140 document area near INCR E
- Operation: Standard increment (add 1) verified
- E-memory handling: Restore required for 0o400-0o1777
- Counter priority: Referenced in section 32-186
- Overflow: Standard 1's complement overflow behavior
- Status: Core behavior verified from PDF context

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference. Counter priority noted but not detailed (see Counter Instructions section).
