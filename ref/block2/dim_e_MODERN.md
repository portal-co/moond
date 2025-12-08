# DIM E — Diminish (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 143-145 (section 32-197).

## Summary

Decrease the magnitude of a value stored at address E, preserving the sign. Positive values decrease, negative values become less negative. Opposite of AUG E.

**Operation:** Decrease magnitude by 1, keeping sign

**Use case:** Magnitude reduction, fine-tuning operations

## Pseudocode

```c
void DIM_E(uint16_t E) {
    // Fetch value from memory
    int16_t value = memory[E];
    
    // Diminish: decrease magnitude preserving sign
    // (1's complement: +0 and -0 are distinct)
    if (value > 0) {
        // Positive: decrement
        value = value - 1;
    } else if (value < 0 && value != 0o77777) {
        // Negative (not -0): increment (less negative)
        value = value + 1;
    } else if (value == 0) {
        // Plus zero: stays +0
        value = 0;
    } else {  // value == 0o77777 (-0)
        // Minus zero: stays -0
        value = 0o77777;
    }
    
    // Store result back to memory
    memory[E] = value & 0o77777;
    
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

### Diminish Behavior

DIM decreases magnitude toward zero while preserving sign:
- **+5** diminished → **+4**
- **+1** diminished → **+0**
- **+0** diminished → **+0** (stays zero)
- **-0** diminished → **-0** (stays zero)
- **-1** diminished → **-0** (magnitude reaches zero)
- **-5** diminished → **-4** (less negative)

### DIM vs INCR

- **DIM E:** Magnitude decrease (preserves sign)
- **INCR E:** Standard addition (+1)

DIM moves values toward zero, INCR adds one.

### 1's Complement Behavior

With 1's complement:
- **+2 (0o00002)** diminished → **+1 (0o00001)**
- **+1 (0o00001)** diminished → **+0 (0o00000)**
- **-1 (0o77776)** diminished → **-0 (0o77777)**
- **-2 (0o77775)** diminished → **-1 (0o77776)**

### Use Cases

1. **Magnitude reduction:** Decrease absolute value
2. **Damping:** Reduce signal strength toward zero
3. **Convergence:** Move values toward equilibrium
4. **Angular adjustments:** Decrease angle magnitude

### AUG E Complement

**AUG E** (Augment) is the opposite: increases magnitude away from zero.

## Citations

- AGCIS Issue 32 (Block-2), pages 143-145
  - Section 32-197: DIM E instruction description
  - Note: DIM appears after DAS section in context
- 1's complement arithmetic: AGCIS Issue 3, pages 12-15

## Audit

- PDF source: AGCIS Issue 32 pages 143-145 document area with DIM E
- Magnitude behavior: "diminishes" means decrease magnitude preserving sign
- Sign preservation: Positive stays positive (or becomes +0), negative stays negative (or becomes -0)
- Zero behavior: Both +0 and -0 stay at zero when diminished
- Status: Core behavior verified from PDF section context

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference. Magnitude operation complementary to AUG E.
