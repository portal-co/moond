#pragma once

#include <stdint.h>

// AGC Block-2 Bit Manipulation Helpers
//
// AGC uses big-endian bit numbering:
//   Bit 1 = MSB (most significant)
//   Bit 15 = LSB (least significant)
//   Bit 0 = Parity bit (in 16-bit words)
//
// C uint16_t uses standard numbering:
//   Bit 15 = MSB
//   Bit 0 = LSB
//
// For 15-bit values (parity stripped):
//   AGC bit N maps to C bit (15-N)
//
// Examples:
//   AGC bit 1 -> C bit 14
//   AGC bit 6 -> C bit 9
//   AGC bit 7 -> C bit 8
//   AGC bit 15 -> C bit 0
//
// CRITICAL: Bit Reversal IS Needed!
// -----------------------------------
// AGC numbers bits BACKWARDS from C:
//   - AGC bit 1 = MSB (most significant)
//   - AGC bit 15 = LSB (least significant)
//
// When the AGC documentation says order code "01" octal is in bits 1-6:
//   - "01" octal = 1 decimal = 0b000001 in normal binary notation
//   - AGC bit 1 (MSB of field) should contain the MSB of value = 0
//   - AGC bit 6 (LSB of field) should contain the LSB of value = 1
//   - In C representation: bit 14 = 0, bit 9 = 1
//   - So C bits 14-9 should be: 0b100000 (reversed!)
//   
// When we mask out bits from C representation, they're in C order (LSB=bit 0).
// But AGC documentation specifies values assuming bit 1 is MSB.
// Therefore: we must reverse multi-bit fields after extraction or before insertion.

// Convert AGC bit number to C bit number (for 15-bit words)
// agc_bit: 1-15 (AGC numbering)
// returns: 0-14 (C numbering)
static inline uint8_t agc_to_c_bit(uint8_t agc_bit) {
    return 15 - agc_bit;
}

// Convert C bit number to AGC bit number (for 15-bit words)
// c_bit: 0-14 (C numbering)
// returns: 1-15 (AGC numbering)
static inline uint8_t c_to_agc_bit(uint8_t c_bit) {
    return 15 - c_bit;
}

// Extract AGC bit range from word
// word: 15-bit value
// start_bit: Starting AGC bit number (1-15, inclusive)
// end_bit: Ending AGC bit number (1-15, inclusive)
// returns: Extracted value (right-justified)
//
// Example: extract_agc_bits(word, 1, 6) extracts AGC bits 1-6 (top 6 bits)
static inline uint16_t extract_agc_bits(uint16_t word, uint8_t start_bit, uint8_t end_bit) {
    uint8_t c_start = agc_to_c_bit(start_bit);  // Higher C bit (more significant)
    uint8_t c_end = agc_to_c_bit(end_bit);      // Lower C bit (less significant)
    uint8_t width = c_start - c_end + 1;
    uint16_t mask = (1 << width) - 1;
    return (word >> c_end) & mask;
}

// Insert value into AGC bit range
// word: 15-bit value to modify
// value: Value to insert
// start_bit: Starting AGC bit number (1-15, inclusive)
// end_bit: Ending AGC bit number (1-15, inclusive)
// returns: Modified word with value inserted
//
// Example: insert_agc_bits(word, 0x3F, 1, 6) inserts value into AGC bits 1-6
static inline uint16_t insert_agc_bits(uint16_t word, uint16_t value, uint8_t start_bit, uint8_t end_bit) {
    uint8_t c_start = agc_to_c_bit(start_bit);
    uint8_t c_end = agc_to_c_bit(end_bit);
    uint8_t width = c_start - c_end + 1;
    uint16_t mask = (1 << width) - 1;
    
    // Clear the bit range
    word &= ~(mask << c_end);
    
    // Insert the value
    word |= (value & mask) << c_end;
    
    return word;
}

// Create word from AGC bit range value
// value: Value to place in bit range
// start_bit: Starting AGC bit number (1-15, inclusive)
// end_bit: Ending AGC bit number (1-15, inclusive)
// returns: Word with value in specified bits, zeros elsewhere
//
// Example: make_agc_bits(0x01, 1, 6) creates word with 0x01 in bits 1-6
static inline uint16_t make_agc_bits(uint16_t value, uint8_t start_bit, uint8_t end_bit) {
    return insert_agc_bits(0, value, start_bit, end_bit);
}

// Reverse bits in an N-bit value
// Used when extracting/inserting multi-bit fields from AGC words
// because AGC numbers bits backwards (bit 1 = MSB, bit 15 = LSB)
static inline uint16_t reverse_bits(uint16_t value, uint8_t num_bits) {
    uint16_t result = 0;
    for (uint8_t i = 0; i < num_bits; i++) {
        result = (result << 1) | (value & 1);
        value >>= 1;
    }
    return result;
}

// Extract value from AGC bit range (WITH bit reversal for AGC semantics)
// Use this when you need the value as AGC documentation specifies it
static inline uint16_t extract_agc_bits_reversed(uint16_t word, uint8_t start_bit, uint8_t end_bit) {
    uint8_t num_bits = end_bit - start_bit + 1;
    uint16_t value = extract_agc_bits(word, start_bit, end_bit);
    return reverse_bits(value, num_bits);
}

// Insert value into AGC bit range (WITH bit reversal for AGC semantics)
// Use this when you have a value as AGC documentation specifies it
static inline uint16_t insert_agc_bits_reversed(uint16_t word, uint16_t value, uint8_t start_bit, uint8_t end_bit) {
    uint8_t num_bits = end_bit - start_bit + 1;
    uint16_t reversed_value = reverse_bits(value, num_bits);
    return insert_agc_bits(word, reversed_value, start_bit, end_bit);
}
