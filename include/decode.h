#pragma once

#include "instr.h"
#include "bits.h"
#include <stdint.h>
#include <stdbool.h>

// AGC Block-2 Instruction Decoder
// Decodes raw 15-bit opcodes (bits 1-15 of AGC word)
// Parity bit (bit 0) is ignored during decode; parity will be synthesized
// during recompilation/assembly.
//
// Three instruction formats:
// - Whole codes: 3-bit opcode (bits 1-3) + 12-bit address (bits 4-15)
// - Quarter codes: 5-bit opcode (bits 1-5) + 3-bit quarter (bits 6-8) + 7-bit address (bits 9-15)
// - Channel: 6-bit opcode (bits 1-6) + 9-bit channel address (bits 7-15)
//
// Reference: ref/block2/OPCODE_ENCODING.md
//
// Uses bits.h for AGC bit numbering conversions

// Decoded instruction (alias for shared structure)
typedef moond_instr moond_decoded_instr;

// Decode a 15-bit AGC instruction word
// word: Bits 1-15 of AGC instruction (bit 0 parity is ignored)
// extend_bit: True if previous instruction was EXTEND
moond_decoded_instr moond_decode_instr(uint16_t word, bool extend_bit);

// Helper: Extract 3-bit opcode (AGC bits 1-3) for whole codes
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_opcode_3(uint16_t word) {
    return extract_agc_bits_reversed(word, 1, 3);
}

// Helper: Extract 5-bit opcode (AGC bits 1-5) for quarter codes
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_opcode_5(uint16_t word) {
    return extract_agc_bits_reversed(word, 1, 5);
}

// Helper: Extract 6-bit opcode (AGC bits 1-6) for channel instructions
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_opcode_6(uint16_t word) {
    return extract_agc_bits_reversed(word, 1, 6);
}

// Helper: Extract quarter code (AGC bits 6-8, 1 octal digit)
// Used by quarter code instructions
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_quarter(uint16_t word) {
    return extract_agc_bits_reversed(word, 6, 8);
}

// Helper: Extract 12-bit address (AGC bits 4-15) for whole codes
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint16_t moond_extract_addr_12(uint16_t word) {
    return extract_agc_bits_reversed(word, 4, 15);
}

// Helper: Extract 9-bit address (AGC bits 7-15) for channel instructions
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint16_t moond_extract_addr_9(uint16_t word) {
    return extract_agc_bits_reversed(word, 7, 15);
}

// Helper: Extract 7-bit address (AGC bits 9-15) for quarter codes
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint16_t moond_extract_addr_7(uint16_t word) {
    return extract_agc_bits_reversed(word, 9, 15);
}
