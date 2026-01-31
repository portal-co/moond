# AGC Block-2 Bit Manipulation Helpers

> Created: 2026-01-31T04:51:43.161Z
>
> Helper functions for converting between AGC and C bit numbering

## Overview

AGC uses big-endian bit numbering where bit 1 is the MSB and bit 15 is the LSB. C uses standard numbering where bit 15 is the MSB and bit 0 is the LSB. These helpers provide a clean API for bit manipulation using AGC bit numbers.

## API

### Bit Number Conversion

```c
uint8_t agc_to_c_bit(uint8_t agc_bit);  // Convert AGC bit 1-15 to C bit 14-0
uint8_t c_to_agc_bit(uint8_t c_bit);    // Convert C bit 14-0 to AGC bit 1-15
```

### Bit Field Extraction

```c
// Extract AGC bits start_bit through end_bit (inclusive, both in AGC numbering)
uint16_t extract_agc_bits(uint16_t word, uint8_t start_bit, uint8_t end_bit);
```

**Examples:**
- `extract_agc_bits(word, 1, 3)` - Extract opcode (top 3 bits)
- `extract_agc_bits(word, 1, 6)` - Extract 6-bit opcode  
- `extract_agc_bits(word, 7, 9)` - Extract quarter code
- `extract_agc_bits(word, 10, 15)` - Extract 6-bit address

### Bit Field Insertion

```c
// Insert value into AGC bit range start_bit through end_bit
uint16_t insert_agc_bits(uint16_t word, uint16_t value, uint8_t start_bit, uint8_t end_bit);

// Create word with value in specified bit range, zeros elsewhere
uint16_t make_agc_bits(uint16_t value, uint8_t start_bit, uint8_t end_bit);
```

**Examples:**
- `make_agc_bits(03, 1, 3)` - Create word 030000 (opcode in bits 1-3)
- `insert_agc_bits(word, 050, 10, 15)` - Insert address into bits 10-15

## Usage in Decoder/Encoder

### Decoder (decode.h)

```c
static inline uint8_t moond_extract_opcode_3(uint16_t word) {
    return extract_agc_bits(word, 1, 3);
}

static inline uint8_t moond_extract_quarter(uint16_t word) {
    return extract_agc_bits(word, 7, 9);
}
```

### Encoder (encode.c)

```c
// Build 3-bit opcode instruction
result.word = make_agc_bits(opcode, 1, 3) | make_agc_bits(address, 4, 15);

// Build quarter code instruction
result.word = make_agc_bits(opcode, 1, 6) | 
              make_agc_bits(quarter, 7, 9) |
              make_agc_bits(address, 10, 15);
```

## Benefits

1. **Clarity**: Code explicitly uses AGC bit numbering from documentation
2. **Correctness**: No manual bit shift calculations to get wrong
3. **Maintainability**: Changes to bit layout only need updates in one place
4. **Documentation**: Bit ranges are self-documenting in the code

## Testing

See `src/bits_test.c` for comprehensive tests verifying:
- Bit number conversion correctness
- Extraction of various bit ranges
- Insertion and construction of words
- Round-trip encode/decode scenarios

All tests pass with real AGC instruction encodings.
