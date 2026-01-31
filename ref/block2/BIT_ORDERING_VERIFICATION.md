# AGC Block-2 Bit Ordering Verification

> Created: 2026-01-31T05:18:42.451Z
>
> Verification that no bit reversal is needed for opcode/operand fields

## Question

When extracting multi-bit fields (like opcodes), do the bits need to be reversed between AGC bit order and C bit order?

## Answer: NO - No Reversal Needed

The current implementation is **correct as-is**. Here's why:

### Understanding AGC vs C Bit Numbering

**AGC Bit Numbering** (big-endian):
- Bit 1 = MSB (most significant bit)
- Bit 15 = LSB (least significant bit)
- Bits numbered 1-15 from left to right

**C Bit Numbering** (standard):
- Bit 15 = MSB
- Bit 0 = LSB  
- Bits numbered 0-15 from right to left

**Conversion Formula**: AGC bit N → C bit (15-N)

### Why No Reversal is Needed

When we extract a multi-bit field:
1. The MSB of the field stays the MSB
2. The LSB of the field stays the LSB
3. The bit significance is preserved

**Example: Order Code "01" (octal)**

From AGC documentation: CCS has order code "01" (octal).

1. **Interpret order code as number**: 01 octal = 1 decimal = 0b000001 binary (6 bits)
2. **Place in AGC bits 1-6**: The binary pattern 0b000001 goes into bits 1-6
3. **Convert to C bits**: AGC bits 1-6 → C bits 14-9
4. **Result**: C bits 14-9 contain 0b000001

**No reversal happens!** The MSB (AGC bit 1 = C bit 14) contains 0, and the LSB (AGC bit 6 = C bit 9) contains 1.

### Octal Notation Encodes Bits Correctly

Octal digits naturally group 3 bits in the correct order:

**Example: Address 050 (octal)**
```
050 octal = 0b 000 101 000
             ↑   ↑   ↑
            MSB     LSB (9 bits)
```

When placed in AGC bits 10-15 (C bits 5-0), this becomes:
```
C bit:     5 4 3   2 1 0
         +-----------+
Value:   | 1 0 1 0 0 0 |  = 050 octal
         +-----------+
           ↑       ↑
          MSB     LSB
```

The bit pattern 101000 in C representation equals 40 decimal = 050 octal. ✓

### Complete Example: CCS E 050

**Instruction**: CCS E with address 050

**Encoding**:
- Order code: 01.0
  - Opcode: 01 octal = 0b000001 (6 bits) → AGC bits 1-6 → C bits 14-9
  - Quarter: 0 octal = 0b000 (3 bits) → AGC bits 7-9 → C bits 8-6
- Address: 050 octal = 0b101000 (6 bits) → AGC bits 10-15 → C bits 5-0

**Complete Word** (15 bits in C representation):
```
C bit:  14 13 12 11 10  9 | 8  7  6 | 5  4  3  2  1  0
AGC bit: 1  2  3  4  5  6 | 7  8  9 |10 11 12 13 14 15
        +------------------+---------+------------------+
Value:  | 0  0  0  0  0  1| 0  0  0 | 1  0  1  0  0  0|
        +------------------+---------+------------------+
          opcode (01 oct)   qtr (0)   address (050 oct)
```

**Result**: 0b 000001_000_101000 = 01050 octal ✓

### Verification Test

```c
// Encode CCS E 050
uint16_t word = (01 << 9) | (0 << 6) | 050;  // Using octal literals
printf("Word: %05o\n", word);  // Prints: 01050 ✓

// Decode it back
uint8_t opcode = (word >> 9) & 0x3F;   // Extract: 01 octal ✓
uint8_t quarter = (word >> 6) & 0x7;   // Extract: 0 octal ✓
uint8_t address = word & 0x3F;         // Extract: 050 octal ✓
```

### Test Results

All encoder/decoder tests pass:
- ✅ 23/23 round-trip tests passing
- ✅ CCS E 050 encodes to 01050 and decodes back correctly
- ✅ READ H 030 encodes to 10030 and decodes back correctly
- ✅ All instruction types encode/decode correctly

## Conclusion

**No bit reversal is needed.** The current implementation in `bits.h` and the encoder/decoder is correct:

1. Octal values from the PDF are interpreted as natural numbers
2. These values are placed in bit fields using standard shift operations
3. The bit significance is preserved (MSB stays MSB, LSB stays LSB)
4. Octal notation naturally represents the correct bit patterns

The confusion might arise from AGC's reversed bit numbering (1-15 instead of 14-0), but this is handled by the bit position conversion, not by reversing the bits within fields.

## References

- AGC Block-2 Instructions PDF (agcis_32_blk2_instructions.pdf), Table 32-2
- Current test results: All 23 encode/decode tests passing
- `bits_test.c`: All bit manipulation tests passing
