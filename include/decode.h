#pragma once

#include "instr.h"
#include <stdint.h>
#include <stdbool.h>

// AGC Block-2 Instruction Decoder
// Decodes raw 15-bit opcodes (bits 1-15 of AGC word)
// Parity bit (bit 0) is ignored during decode; parity will be synthesized
// during recompilation/assembly.
//
// Reference: ref/block2/OPCODE_ENCODING.md
//
// AGC Word Format (16 bits including parity):
//   Bit 0: Parity (ignored in this decoder)
//   Bits 1-15: Instruction/data (15 bits)
//
// AGC Bit Numbering vs C uint16_t:
//   AGC uses bit 1 = MSB, bit 15 = LSB
//   In uint16_t: bit 15 = MSB, bit 0 = LSB
//   So AGC bit N maps to C bit (15-N) when parity is stripped
//  
// For a 15-bit value in uint16_t (parity already removed):
//   AGC bit 1 = C bit 14
//   AGC bit 15 = C bit 0
//
// Instruction Encoding:
//   Bits 1-3 (C bits 14-12): Primary opcode (1 octal digit)
//   Bits 4-15 (C bits 11-0): Address/operand (4 octal digits)
//  
// For quarter codes:
//   Bits 1-6 (C bits 14-9): Primary opcode (2 octal digits)
//   Bits 7-9 (C bits 8-6): Quarter code (1 octal digit)
//   Bits 10-15 (C bits 5-0): Address/operand (2 octal digits)

// Decoded instruction (alias for shared structure)
typedef moond_instr moond_decoded_instr;

// Decode a 15-bit AGC instruction word
// word: Bits 1-15 of AGC instruction (bit 0 parity is ignored)
// extend_bit: True if previous instruction was EXTEND
moond_decoded_instr moond_decode_instr(uint16_t word, bool extend_bit);

// Helper: Extract primary opcode (AGC bits 1-3, 1 octal digit)
// For 15-bit word in uint16_t: bits 14-12 (C numbering)
static inline uint8_t moond_extract_opcode_3(uint16_t word) {
    return (word >> 12) & 0x7;  // Top 3 bits
}

// Helper: Extract 6-bit opcode for quarter codes (AGC bits 1-6, 2 octal digits)
// For 15-bit word in uint16_t: bits 14-9 (C numbering)
static inline uint8_t moond_extract_opcode_6(uint16_t word) {
    return (word >> 9) & 0x3F;  // Top 6 bits
}

// Helper: Extract quarter code (AGC bits 7-9, 1 octal digit)
// For 15-bit word in uint16_t: bits 8-6 (C numbering)  
static inline uint8_t moond_extract_quarter(uint16_t word) {
    return (word >> 6) & 0x7;  // Bits 8-6
}

// Helper: Extract 12-bit address (AGC bits 4-15)
// For 15-bit word in uint16_t: bits 11-0 (C numbering)
static inline uint16_t moond_extract_addr_12(uint16_t word) {
    return word & 0x0FFF;  // Bottom 12 bits
}

// Helper: Extract 9-bit channel address (AGC bits 7-15)
// For 15-bit word in uint16_t: bits 8-0 (C numbering)
static inline uint16_t moond_extract_addr_9(uint16_t word) {
    return word & 0x01FF;  // Bottom 9 bits
}
