// AGC Block-2 Opcode Decoder - Design Document
// 
// Parity Note: Parity bits will be synthesized during recompilation/assembly.
// The decoder works with 15-bit instruction words (parity stripped).
//
// AGC Instruction Encoding (from OPCODE_ENCODING.md)
// ==================================================
//
// AGC represents instructions in octal (base-8) naturally because:
// - Each octal digit = 3 bits
// - 15-bit instruction = 5 octal digits
//
// Example: TC 00100 (octal)
//   00100 (octal) = 00000001000000 (binary, 15 bits)
//   
//   Breaking down by octal digits:
//   0  0  1  0  0
//   |  |__|__|__|
//   |     address (12 bits = 4 octal digits)
//   opcode (3 bits = 1 octal digit)
//
// Standard Encoding:
//   Bits 1-3:   Primary opcode (1 octal digit, 3 bits)
//   Bits 4-15:  Address field (4 octal digits, 12 bits)
//
// Quarter Code Encoding:
//   Bits 1-6:   Primary opcode (2 octal digits, 6 bits)
//   Bits 7-9:   Quarter code (1 octal digit, 3 bits)
//   Bits 10-15: Address field (2 octal digits, 6 bits)
//
// Bit Numbering Convention:
//   AGC documentation: Bit 1 = MSB, Bit 15 = LSB
//   C uint16_t: Bit 15 = MSB, Bit 0 = LSB
//
// When storing a 15-bit AGC word in uint16_t:
//   AGC bit N -> C bit (14 - N + 1) = C bit (15 - N)
//   
//   AGC bit 1  (MSB) -> C bit 14
//   AGC bit 15 (LSB) -> C bit 0
//
// Example Conversion: CA 00050 (order code 03., address 00050)
//   
//   Octal representation: 30050
//     3 = opcode (03 octal = 011 binary)
//     0050 = address (00050 octal = 000000101000 binary)
//   
//   Binary layout (AGC bits 1-15):
//     011 000000101000
//     ^^^ ^^^^^^^^^^^^
//     |   address (12 bits)
//     opcode (3 bits)
//   
//   In uint16_t (15-bit value, bit 14 down to bit 0):
//     0b011_000000101000 = 0x3028
//   
//   Extraction:
//     opcode = (word >> 12) & 0x7    // Get top 3 bits
//     addr   = word & 0xFFF           // Get bottom 12 bits
//
// For whole order codes (bits 1-9 define instruction):
//   Example: 10.0 (READ H 030)
//     Octal: 40030
//     Binary: 100_000_000011000
//             ^^^_^^^_^^^^^^^^^^
//             |   |   channel (9 bits)
//             |   quarter (3 bits)
//             primary (3 bits)
//   
//   Extraction:
//     primary_opcode = (word >> 12) & 0x7    // bits 14-12
//     quarter_code   = (word >> 9) & 0x7      // bits 11-9
//     channel_addr   = word & 0x1FF           // bits 8-0
//
// Octal String to uint16_t Conversion:
//   Input: "30050"
//   Process: Treat as base-8 number
//   Output: 0x3028
//
//   Verification:
//     3*8^4 + 0*8^3 + 0*8^2 + 5*8^1 + 0*8^0
//     = 3*4096 + 0 + 0 + 40 + 0
//     = 12288 + 40
//     = 12328 (decimal)
//     = 0x3028 (hex)
//
// This file documents the encoding for reference during implementation.
