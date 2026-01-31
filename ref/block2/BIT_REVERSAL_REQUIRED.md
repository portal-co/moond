# AGC Block-2 Bit Reversal Requirement

> Created: 2026-01-31T05:20:00.000Z
>
> **CRITICAL**: Multi-bit field values MUST be reversed when encoding/decoding AGC instructions

## The Problem

AGC uses **reversed bit numbering** compared to modern C:

| System | MSB Position | LSB Position | Direction |
|--------|--------------|--------------|-----------|
| AGC | Bit 1 | Bit 15 | Left→Right (big-endian) |
| C uint16_t | Bit 15 | Bit 0 | Right→Left (standard) |

## Why Reversal is Needed

When AGC documentation specifies order code "01" (octal) in bits 1-6:

1. **"01" octal** = 1 decimal = **0b000001** in standard binary notation
2. In **AGC bit numbering**:
   - Bit 1 (MSB of 6-bit field) should contain **MSB of value** = 0
   - Bit 6 (LSB of 6-bit field) should contain **LSB of value** = 1
3. In **C bit storage** (after position mapping AGC bit N → C bit 15-N):
   - C bit 14 (= AGC bit 1) should contain 0
   - C bit 9 (= AGC bit 6) should contain 1
   - **But**: if we just shift the value 0b000001 left by 9, we get:
     - C bit 14 = 0, C bit 9 = 1 ✓ **WAIT, this seems correct!**

## RE-ANALYSIS

Let me reconsider. When we have value 0b000001:
- In binary string notation: "000001" (MSB left, LSB right)
- As a C integer shifted left 9: puts LSB at bit 9, MSB at bit 14
- AGC bit 1 = C bit 14 gets the MSB (leftmost) of our binary string = 0 ✓
- AGC bit 6 = C bit 9 gets the LSB (rightmost) of our binary string = 1 ✓

This suggests NO REVERSAL is needed for the bit pattern itself!

## The Real Issue: Octal Digit Ordering

The reversal might be needed for **octal digit interpretation**, not bit patterns!

When AGC documentation writes "01" in octal:
- First digit (left): 0
- Second digit (right): 1
- As a number: 0×8 + 1 = 1 decimal = 0b000001

This is standard octal interpretation, no reversal needed.

## Conclusion Pending

Need to verify against actual AGC machine code or assembler output to determine if bit reversal is actually required. The theoretical analysis suggests it may NOT be needed, but empirical evidence (opcode collisions mentioned) suggests it IS needed.

**Action**: Implement both versions and test against known AGC binary programs.
