# Bit Reversal Decision for AGC Block-2

> Date: 2026-01-31  
> Status: **IMPLEMENTING BIT REVERSAL**  
> Reason: Empirical evidence of opcode collisions without reversal

## Decision

Multi-bit field values (opcodes, quarters, addresses) **MUST be bit-reversed** when encoding/decoding AGC Block-2 instructions.

## Rationale

1. **AGC bit numbering is reversed**: Bit 1 = MSB, Bit 15 = LSB (opposite of C)
2. **Opcode collisions observed**: Without bit reversal, different instructions produce overlapping bit patterns
3. **AGC hardware expectation**: The physical AGC hardware interprets bits in AGC order (1-15), not C order (15-0)

## Implementation

Use these functions from `bits.h`:
- `extract_agc_bits_reversed()` - Extract multi-bit fields with reversal
- `insert_agc_bits_reversed()` - Insert multi-bit fields with reversal
- `reverse_bits(value, num_bits)` - Reverse N-bit value

## Example

Order code "01" octal in AGC bits 1-6:
- Value: 01 octal = 1 decimal = 0b000001
- Reversed (6 bits): 0b100000 = 040 octal = 32 decimal
- Stored in C bits 14-9: (040 << 9)
- Encoded word: CCS E 050 = 40050 octal (NOT 01050)

## Cross-Reference

- `include/bits.h` - Bit manipulation with reversal
- `src/decode.c` - Decoder using reversed extraction
- `src/encode.c` - Encoder using reversed insertion
- `src/bits_test.c` - Tests for bit reversal functions
