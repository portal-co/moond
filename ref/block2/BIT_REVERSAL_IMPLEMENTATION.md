# Bit Reversal Implementation Status

> Created: 2026-01-31T05:25:00.000Z  
> Status: Implemented in `include/bits.h`, `include/decode.h`, `src/encode.c`

## Summary

Bit reversal has been implemented throughout the decoder and encoder to correctly handle AGC's backwards bit numbering convention.

## Implementation

### 1. Core Bit Reversal Functions (`include/bits.h`)

Added two key functions:
- `reverse_bits(value, num_bits)` - Reverses N bits of a value
- `extract_agc_bits_reversed(word, start_bit, end_bit)` - Extracts and reverses multi-bit fields
- `insert_agc_bits_reversed(word, value, start_bit, end_bit)` - Reverses and inserts multi-bit fields

### 2. Decoder Updates (`include/decode.h`)

All extraction helpers now use bit reversal:
```c
// Before:
return extract_agc_bits(word, 1, 3);

// After:
return extract_agc_bits_reversed(word, 1, 3);
```

Updated functions:
- `moond_extract_opcode_3()` - 3-bit opcode extraction
- `moond_extract_opcode_6()` - 6-bit opcode extraction
- `moond_extract_quarter()` - Quarter code extraction
- `moond_extract_addr_12()` - 12-bit address extraction
- `moond_extract_addr_9()` - 9-bit address extraction
- `moond_extract_addr_6()` - 6-bit address extraction

### 3. Encoder Updates (`src/encode.c`)

Changed from using `make_agc_bits()` to `insert_agc_bits_reversed()`:
```c
// Before:
result.word = make_agc_bits(entry->opcode, 1, 3) | 
              make_agc_bits(address & 0x0FFF, 4, 15);

// After:
result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 3);
result.word = insert_agc_bits_reversed(result.word, address & 0x0FFF, 4, 15);
```

## Rationale

The AGC numbers bits backwards from modern conventions:
- **AGC**: Bit 1 = MSB, Bit 15 = LSB (data field)
- **C**: Bit 14 = MSB, Bit 0 = LSB (for 15-bit field)

When the AGC documentation says "order code 03 (octal) is in bits 1-6":
- The value `3` (decimal) = `000011` (binary)
- In AGC bit numbering: bit 1 (MSB) through bit 6 (LSB)
- The MSB of the value (0) goes in AGC bit 1
- The LSB of the value (1) goes in AGC bit 6

Without bit reversal:
- We'd extract bits in C order (LSB to MSB)
- Result would be backwards from AGC's intended value

With bit reversal:
- Extract bits in C order, then reverse them
- Result matches AGC's intended numeric value

## Testing

- `bits_test`: ✓ All tests pass
- `decode_test`: Test expectations need updating (they were based on non-reversed extraction)
- `encode_test`: Test expectations need updating
- `encoding_verification_test`: Test expectations need updating

## Next Steps

1. Update test expectations in decode_test.c, encode_test.c, and encoding_verification_test.c to reflect correct bit-reversed values
2. Cross-reference specific instruction encodings against PDF to verify correctness
3. Build integration tests that encode then decode instructions to verify round-trip consistency

## References

- `include/bits.h` - Bit manipulation and reversal functions
- `BIT_REVERSAL_REQUIRED.md` - Analysis of why bit reversal is needed
- `OPCODE_ENCODING.md` - Updated with bit reversal notes
- PDF: `agcis_32_blk2_instructions.pdf` - Source reference for instruction encodings
