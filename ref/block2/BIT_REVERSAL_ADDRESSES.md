# AGC Bit Reversal and Address Encoding

> Created: 2026-01-31T06:18:42.183Z
>
> This document clarifies the critical distinction between AGC address notation and binary word encoding

## The Bit Reversal Problem

The AGC uses **backwards bit numbering** where bit 1 is the MSB and bit 15 is the LSB. This creates confusion when encoding/decoding addresses.

### Example: TC to address 04000 (octal)

When the AGC documentation says "TC 04000", it means:
- **Opcode**: 00 (3 bits in AGC bits 1-3)
- **Address**: 04000 octal = 100000000000 binary (12 bits in AGC bits 4-15)

**In AGC bit order** (bit 1 = MSB, reading left to right):
```
Bit:  1  2  3 | 4  5  6  7  8  9 10 11 12 13 14 15
     0  0  0 | 1  0  0  0  0  0  0  0  0  0  0  0
     \_____/   \___________________________________/
      opcode              address 04000
```

**But in C representation** (bit 14 = MSB of 15-bit field):
```
C bit: 14 13 12 |11 10  9  8  7  6  5  4  3  2  1  0
        0  0  0 | 0  0  0  0  0  0  0  0  0  0  0  1
```

This is **hex 0x0001**, NOT 0x0800!

### The Reversal

The address bits need to be **bit-reversed** when converting between AGC notation and binary word encoding:

- AGC address 04000 octal = 100000000000 binary (AGC bit order, MSB first)
- After bit reversal = 000000000001 binary (C bit order, LSB first)  
- = Hex 0x0001

### Common Confusion

- **C octal literal `04000`** = decimal 2048 = hex 0x0800
- **AGC address "04000"** (after bit reversal) = hex 0x0001

These are NOT the same!

## Practical Examples

### READ H Channel 0

**AGC Notation**: Order code 010.0 with channel address 00
- Opcode 6-bit: 010 octal = 001000 binary  
- Quarter: 0 octal = 000 binary
- Channel: 00 octal = 000000 binary

**Binary word** (AGC bits 1-15, MSB first):
```
001 000 000 000 000
```

**After bit reversal for C storage**:
```
000 000 000 000 100 (reversed)
```

**Result**: Hex 0x0800 (octal 004000 in C notation)

### TC to Fixed Memory Start

**AGC Notation**: TC 04000 (GO instruction restart address)
- Opcode: 00
- Address: 04000 octal

**Binary word after bit reversal**: Hex 0x0001

### The Key Rule

When working with AGC addresses in documentation:
1. **Don't use C octal literals directly** (e.g., `04000` in C)
2. **Use `insert_agc_bits_reversed()`** to encode AGC addresses
3. **Use `extract_agc_bits_reversed()`** to decode AGC addresses
4. The hex/octal you see in memory dumps is **already bit-reversed**

## Memory Map Implications

Given bit reversal, the memory map in hex words is:

| Hex Range    | C Octal    | AGC Address Range | Description |
|--------------|------------|-------------------|-------------|
| 0x0000       | 000000     | 00000            | Register A  |
| 0x0001       | 000001     | 04000            | Fixed mem start (GO addr) |
| 0x0002       | 000002     | 02000            | - |
| 0x0008-0x07FF| 000010-003777 | Varied        | Erasable memory (reversed) |
| 0x0800       | 004000     | READ 010.0       | Channel instruction space |

The collision between TC and READ H occurs because:
- **Hex 0x0800** matches READ H (010.0) - this is CORRECT
- **Hex 0x0001** matches TC 04000 or GO - this is ALSO CORRECT
- **They don't actually collide** - they're different words!

## Decoder Validation

The decoder should validate:
1. TC is only valid for **reasonable jump targets** in fixed memory
2. Since addresses >= 04000 (AGC notation) correspond to words around 0x0001-0x0FFF after reversal
3. TC to register addresses (0x0000-0x0007 after appropriate reversal) should be rejected as nonsensical

However, with bit reversal, the simple check "addr >= 04000" in C octal doesn't work correctly. We need to check the **decoded AGC address value**, not the raw bits.
