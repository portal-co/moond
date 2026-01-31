#pragma once

#include "instr.h"
#include <stdint.h>
#include <stdbool.h>

// AGC Block-2 Instruction Encoder
// Encodes instruction structures into 15-bit machine code
// Parity bit (bit 0) will be synthesized during final assembly stage
//
// Reference: ref/block2/OPCODE_ENCODING.md

// Encode an instruction to a 15-bit word (no parity)
// The instruction structure specifies type, address, and mode
moond_encode_result moond_encode_instr(const moond_instr* instr);

// Encode an instruction with simplified parameters
// Automatically determines address mode and other fields from type
moond_encode_result moond_encode_simple(moond_instr_type type, uint16_t address);

// Helper: Validate address range for instruction type
bool moond_validate_address(moond_instr_type type, uint16_t address);

// Helper: Get maximum address value for instruction type
uint16_t moond_max_address(moond_instr_type type);
