#pragma once

#include <stdint.h>
#include <stdbool.h>

// AGC Block-2 Instruction Types and Structures
// Shared between decoder and encoder
//
// Reference: ref/block2/OPCODE_ENCODING.md
//
// Parity Note: Parity bit (bit 0) is synthesized during encoding/assembly
// and ignored during decoding. The encoder will generate 15-bit words
// without parity; parity synthesis happens at a later stage.

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
    INSTR_MP,      // Multiply by K (extracode)
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

// Instruction structure (used by both encoder and decoder)
typedef struct {
    moond_instr_type type;
    moond_addr_mode addr_mode;
    uint16_t address;          // 12-bit address (K/E/F) or 9-bit channel (H)
    bool requires_extend;      // True if this instruction requires EXTEND prefix
    bool is_extracode;         // True if decoded with extend_bit set
    uint8_t opcode;            // Raw opcode bits (bits 1-3 or 1-6)
    uint8_t quarter_code;      // Quarter code (bits 7-9) if applicable, 0xff otherwise
    int8_t status;             // Number of valid instructions recognized minus one (two's complement)
                               // -1 (0xFF) = no match, 0 = one match, 1 = two matches, etc.
} moond_instr;

// Encoding result
typedef struct {
    bool success;
    uint16_t word;             // Encoded 15-bit instruction (no parity)
    const char* error;         // Error message if !success
} moond_encode_result;

// Get instruction mnemonic string
const char* moond_instr_mnemonic(moond_instr_type type);

// Get address mode string
const char* moond_addr_mode_str(moond_addr_mode mode);

// Helper: Check if instruction requires EXTEND prefix
bool moond_instr_needs_extend(moond_instr_type type);
