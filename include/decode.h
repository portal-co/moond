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
// Reference: ref/block2/OPCODE_ENCODING.md
//
// Uses bits.h for AGC bit numbering conversions

// Decoded instruction (alias for shared structure)
typedef moond_instr moond_decoded_instr;

// Decode a 15-bit AGC instruction word
// word: Bits 1-15 of AGC instruction (bit 0 parity is ignored)
// extend_bit: True if previous instruction was EXTEND
moond_decoded_instr moond_decode_instr(uint16_t word, bool extend_bit);

// Helper: Extract primary opcode (AGC bits 1-3, 1 octal digit)
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_opcode_3(uint16_t word) {
    return extract_agc_bits_reversed(word, 1, 3);
}

// Helper: Extract 6-bit opcode for quarter codes (AGC bits 1-6, 2 octal digits)
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_opcode_6(uint16_t word) {
    return extract_agc_bits_reversed(word, 1, 6);
}

// Helper: Extract quarter code (AGC bits 7-9, 1 octal digit)
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint8_t moond_extract_quarter(uint16_t word) {
    return extract_agc_bits_reversed(word, 7, 9);
}

// Helper: Extract 12-bit address (AGC bits 4-15)
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint16_t moond_extract_addr_12(uint16_t word) {
    return extract_agc_bits_reversed(word, 4, 15);
}

// Helper: Extract 9-bit channel address (AGC bits 7-15)
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint16_t moond_extract_addr_9(uint16_t word) {
    return extract_agc_bits_reversed(word, 7, 15);
}

// Helper: Extract 6-bit address (AGC bits 10-15) for quarter codes
// Note: Uses bit reversal to get correct AGC numeric value
static inline uint16_t moond_extract_addr_6(uint16_t word) {
    return extract_agc_bits_reversed(word, 10, 15);
}
