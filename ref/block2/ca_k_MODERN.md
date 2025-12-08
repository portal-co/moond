# CA K — Clear and Add (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 67-68 (section 32-67).

## Summary

Load a value from memory address K into the accumulator. Simple data transfer instruction.

**Operation:** `A = memory[K]`

**Use case:** Load values, initialize accumulator, data movement

## Pseudocode

```c
void CA_K(uint16_t K) {
    // Clear accumulator and load from memory
    A = memory[K] & 0o77777;  // 15-bit value
    
    // Branch to next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    Z = Z + 1;
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);
}
```

## Notes

### Clear and Add

The name "Clear and Add" reflects the operation:
1. **Clear:** Zero out the accumulator
2. **Add:** Add value from memory (A = 0 + memory[K] = memory[K])

Effectively: **A = memory[K]**

### Comparison with Other Load Instructions

- **CA K:** Load from basic memory (K-type addressing)
- **XCH E:** Exchange with extended memory (loads and stores)
- **DCA K:** Load double precision (loads two words into A and L)

### Use Cases

1. **Load constants:** `CA CONSTANT` loads a constant value
2. **Initialize:** Start computations with a known value
3. **Data movement:** Transfer data to accumulator for processing
4. **Reset:** Clear accumulator by loading zero

### Example

```c
// Load a value and perform operations
CA ANGLE        // Load angle from memory
AD CORRECTION   // Add correction
TS NEWANGLE     // Store result
```

### Block-2 vs Block-1

CA K behavior is identical in Block-1 and Block-2. Both use basic K-type addressing.

## Citations

- AGCIS Issue 32 (Block-2), pages 67-68
  - Section 32-67: CA K instruction description
  - Fetching and Storing Instructions section
- Basic addressing: AGCIS Issue 2, pages 15-20 (K-type addresses)

## Audit

- PDF source: AGCIS Issue 32 pages 67-68 document CA K operation
- Operation: Simple load from memory into accumulator
- Addressing: K-type (basic 12-bit address)
- Block compatibility: Same behavior in Block-1 and Block-2
- Status: Core behavior verified from PDF source

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference. Simple, clear data load operation.
