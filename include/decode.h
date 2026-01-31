#pragma once

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

// Instruction type tags
typedef enum {
    // Sequence changing
    INSTR_TC,      // Transfer Control to K
    INSTR_TCF,     // Transfer Control to Fixed F
    INSTR_CCS,     // Count, Compare, and Skip on E (extracode)
    INSTR_BZF,     // Branch on Zero to Fixed F
    INSTR_BZMF,    // Branch on Zero or Minus to Fixed F
    
    // Fetching and storing
    INSTR_CA,      // Clear and Add K
    INSTR_CS,      // Clear and Subtract K
    INSTR_DCA,     // Double Clear and Add K
    INSTR_DCS,     // Double Clear and Subtract K
    INSTR_TS,      // Transfer to Storage E (extracode)
    INSTR_XCH,     // Exchange A and E (extracode)
    INSTR_LXCH,    // Exchange L and E (extracode)
    INSTR_QXCH,    // Exchange Q and E (extracode)
    INSTR_DXCH,    // Double Exchange A and E (extracode)
    
    // Modifying
    INSTR_NDX,     // Index with E/K
    
    // Arithmetic and logic
    INSTR_AD,      // Add K
    INSTR_SU,      // Subtract E (extracode)
    INSTR_MP,      // Multiply by K
    INSTR_DV,      // Divide by E (extracode)
    INSTR_ADS,     // Add to Storage E (extracode)
    INSTR_DAS,     // Double Add to Storage E (extracode)
    INSTR_INCR,    // Increment E (extracode)
    INSTR_AUG,     // Augment E (extracode)
    INSTR_DIM,     // Diminish E (extracode)
    INSTR_MSU,     // Modular Subtract E (extracode)
    INSTR_MSK,     // Mask with K
    
    // Channel
    INSTR_READ,    // Read H
    INSTR_WRITE,   // Write H
    INSTR_RAND,    // Read and AND H
    INSTR_WAND,    // Write and AND H (extracode)
    INSTR_ROR,     // Read and OR H
    INSTR_WOR,     // Write and OR H (extracode)
    INSTR_RXOR,    // Read and Exclusive OR H (extracode)
    
    // Special
    INSTR_EXTEND,  // Extend (enables extracode)
    INSTR_INHINT,  // Inhibit Interrupt
    INSTR_RELINT,  // Release Inhibit Interrupt
    INSTR_RESUME,  // Resume Interrupted Program
    
    // Involuntary
    INSTR_GO,      // Go (restart at E-memory 04000)
    
    // Counter (no explicit encoding - triggered by hardware)
    INSTR_PINC,    // Plus Increment C
    INSTR_MINC,    // Minus Increment C
    INSTR_DINC,    // Diminish Increment C
    INSTR_PCDU,    // Plus Counter Down Up C
    INSTR_MCDU,    // Minus Counter Down Up C
    INSTR_SHINC,   // Shift Increment C
    INSTR_SHANC,   // Shift and Add Increment C
    
    // Peripheral (controlled by GSE)
    INSTR_TCSAJ,   // Transfer Control to Specified Address K
    INSTR_FETCH,   // Fetch K
    INSTR_STORE,   // Store E (extracode)
    INSTR_INOTRD,  // I/O Not Read H
    INSTR_INOTLD,  // I/O Not Load H
    
    INSTR_UNKNOWN  // Unknown/invalid instruction
} moond_instr_type;

// Address mode types
typedef enum {
    ADDR_K,        // K-type: 12-bit address (CP/E-memory/F-memory)
    ADDR_E,        // E-type: 12-bit address (CP/E-memory)
    ADDR_F,        // F-type: 12-bit address (F-memory only)
    ADDR_H,        // H-type: 9-bit I/O channel address
    ADDR_C,        // C-type: counter address
    ADDR_NONE      // No address field
} moond_addr_mode;

// Decoded instruction
typedef struct {
    moond_instr_type type;
    moond_addr_mode addr_mode;
    uint16_t address;          // 12-bit address (K/E/F) or 9-bit channel (H)
    bool requires_extend;      // True if this instruction requires EXTEND prefix
    bool is_extracode;         // True if decoded with extend_bit set
    uint8_t opcode;            // Raw opcode bits (bits 1-3 or 1-6)
    uint8_t quarter_code;      // Quarter code (bits 7-9) if applicable, 0xff otherwise
} moond_decoded_instr;

// Decode a 15-bit AGC instruction word
// word: Bits 1-15 of AGC instruction (bit 0 parity is ignored)
// extend_bit: True if previous instruction was EXTEND
moond_decoded_instr moond_decode_instr(uint16_t word, bool extend_bit);

// Get instruction mnemonic string
const char* moond_instr_mnemonic(moond_instr_type type);

// Get address mode string
const char* moond_addr_mode_str(moond_addr_mode mode);

// Helper: Check if instruction requires EXTEND prefix
bool moond_instr_needs_extend(moond_instr_type type);

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
