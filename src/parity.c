#include "parity.h"

// Count number of 1-bits in a word (population count)
static uint8_t popcount(uint16_t word) {
    uint8_t count = 0;
    while (word) {
        count += word & 1;
        word >>= 1;
    }
    return count;
}

uint8_t moond_calculate_parity(uint16_t word_15bit) {
    // Mask to 15 bits
    word_15bit &= 0x7FFF;
    
    // Count 1-bits
    uint8_t ones = popcount(word_15bit);
    
    // Return 1 if count is even (to make total odd), 0 if count is odd
    return (ones & 1) ? 0 : 1;
}

uint16_t moond_add_parity(uint16_t word_15bit) {
    // Mask to 15 bits
    word_15bit &= 0x7FFF;
    
    // Calculate parity bit
    uint8_t parity = moond_calculate_parity(word_15bit);
    
    // Create 16-bit word: parity in bit 0 (LSB), instruction in bits 1-15
    // In AGC numbering: bit 0 = parity (LSB in C), bits 1-15 = instruction (MSB in C)
    return (word_15bit << 1) | parity;
}

bool moond_verify_parity(uint16_t word_16bit) {
    // Count all 16 bits
    uint8_t ones = popcount(word_16bit);
    
    // Parity is correct if total count is odd
    return (ones & 1) == 1;
}
