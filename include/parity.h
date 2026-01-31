#pragma once

#include <stdint.h>
#include <stdbool.h>

// AGC Block-2 Parity Synthesis
// Synthesizes odd parity bit for 15-bit AGC words
//
// AGC uses odd parity: the total number of 1-bits in the 16-bit word
// (including parity bit 0) must be odd.

// Calculate odd parity for a 15-bit word
// Returns 0 or 1 to make the total bit count odd
uint8_t moond_calculate_parity(uint16_t word_15bit);

// Add parity bit to a 15-bit word to create a 16-bit word with parity
// Bit 0 (LSB in C, AGC parity bit) will be set appropriately
uint16_t moond_add_parity(uint16_t word_15bit);

// Verify parity of a 16-bit word with parity
// Returns true if parity is correct (odd parity)
bool moond_verify_parity(uint16_t word_16bit);

// Strip parity bit from a 16-bit word
// Returns the 15-bit instruction/data value
static inline uint16_t moond_strip_parity(uint16_t word_16bit) {
    return (word_16bit >> 1) & 0x7FFF;
}
