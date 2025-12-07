# Instruction type (AGC documentation)

This file documents the "Instruction" documentation type used across the per-instruction Markdown files in ref/block2/ and ref/instr/.

Concept
- An Instruction in these docs is a parsed/normalized view of the 15/16-bit word stored in memory that the AGC executes. It separates the raw word into an order code (opcode), address/operand, and preserves the raw_word for reference.

C-like type example

// Logical representation used in pseudocode docs
typedef struct Instruction {
    uint16_t raw_word;     // raw 15/16-bit word as read from memory (preserves sign/overflow bits)
    uint16_t order_code;   // extracted order code (bits EXT..13 or EXT..10 depending on quarter code)
    uint16_t address;      // effective address portion (10 or 12 bits depending on instruction type)
} Instruction;

Helpers (documented semantics)
- fetch_instruction(addr): returns Instruction parsed from memory location `addr` (fills raw_word, order_code, address).
- derive_instruction(base_inst, idx): returns a new Instruction obtained by adding `idx` to the address portion of `base_inst` following AGC indexing rules (handles EXT/quarter-code, wrap, and order-code adjustment).
- extract_order_code(raw_word): extracts the order code bits and EXT bit according to the AGC encoding used in this repo.
- sign_extend15(v): converts a 15-bit AGC value into a signed 32-bit integer for arithmetic in pseudocode.

Notes
- All per-instruction pseudocode in ref/block2/ and ref/instr/ should use this Instruction representation for clarity and consistency.
- The EXT/quarter-code rules and exact bit numbering are documented in the central processor notes (see ref/cpu/registers.md) and should be used by helpers to ensure accurate emulation semantics.
