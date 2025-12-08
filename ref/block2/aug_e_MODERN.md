# AUG E — Augment (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 141-143 (section 32-192).

## Summary

Increment the magnitude of a value stored at address E, preserving the sign. Positive values increase, negative values become more negative.

**Operation:** Increase magnitude by 1, keeping sign

**Use case:** Angular adjustments, fine-tuning computations

## Pseudocode

```c
void AUG_E(uint16_t E) {
    // Fetch value from memory
    int16_t value = memory[E];
    
    // Augment: increase magnitude preserving sign
    // (1's complement: +0 and -0 are distinct)
    if (value > 0) {
        // Positive: increment
        value = value + 1;
    } else if (value < 0 && value != 0o77777) {
        // Negative (not -0): decrement (more negative)
        value = value - 1;
    } else if (value == 0) {
        // Plus zero: becomes +1
        value = 1;
    } else {  // value == 0o77777 (-0)
        // Minus zero: becomes -1
        value = 0o77776;  // -1 in 1's complement
    }
    
    // Store result back to memory
    memory[E] = value & 0o77777;
    
    // Restore E-memory if needed
    if (E >= 0o400 && E <= 0o1777) {
        // E-memory restore already done by write
    }
    
    // Branch to next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    Z = Z + 1;
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);
}
```

## Notes

### Augment vs Increment

- **INCR E:** Adds 1 (standard addition: -1 + 1 = 0)
- **AUG E:** Increases magnitude (+1 becomes +2, -1 becomes -2)

AUG preserves sign while increasing distance from zero.

### 1's Complement Behavior

With 1's complement:
- **+0 (0o00000)** augmented → **+1 (0o00001)**
- **+1 (0o00001)** augmented → **+2 (0o00002)**
- **-0 (0o77777)** augmented → **-1 (0o77776)**
- **-1 (0o77776)** augmented → **-2 (0o77775)**

### Use Cases

1. **Angular resolution:** Increase angle magnitude
2. **Fine adjustments:** Tweak navigation parameters
3. **Magnitude operations:** Operations on absolute values

### DIM E Complement

**DIM E** (Diminish) is the opposite: decreases magnitude toward zero.

## Citations

- AGCIS Issue 32 (Block-2), pages 141-143
  - Section 32-192: AUG E instruction description
- 1's complement arithmetic: AGCIS Issue 3, pages 12-15

## Audit

- PDF source: AGCIS Issue 32 pages 141-143 document AUG E operation
- Magnitude behavior: "augments" means increase magnitude preserving sign
- Sign preservation: Positive stays positive, negative stays negative
- 1's complement: +0/-0 handling verified from section 32-192
- Status: Core behavior verified from PDF source

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference. 1's complement +0/-0 cases shown explicitly.
