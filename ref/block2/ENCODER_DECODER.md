# AGC Block-2 Encoder/Decoder Implementation

> Created: 2026-01-31T04:33:21.162Z
>
> Implementation of instruction encoding and decoding for AGC Block-2
> Reference: ref/block2/OPCODE_ENCODING.md

## Architecture

### Shared Components (`include/instr.h`, `src/instr.c`)

- **`moond_instr_type`**: Enum of all instruction types
- **`moond_addr_mode`**: Enum of address modes (K, E, F, H, C, NONE)
- **`moond_instr`**: Tagged union structure shared by encoder and decoder
- Utility functions: `moond_instr_mnemonic()`, `moond_addr_mode_str()`, `moond_instr_needs_extend()`

### Decoder (`include/decode.h`, `src/decode.c`)

**Function**: `moond_decoded_instr moond_decode_instr(uint16_t word, bool extend_bit)`

Decodes 15-bit AGC instructions into structured format:
- Extracts opcode (3-bit or 6-bit)
- Extracts quarter codes where applicable  
- Extracts address fields (9-bit or 12-bit depending on instruction)
- Identifies whether EXTEND prefix is required
- Returns tagged union with instruction type, address mode, and operands

**Key Features**:
- Uses OCTAL() macro from core.h for clarity
- Handles basic vs extracode mode distinction
- Supports all address modes (K, E, F, H)
- Validates special fixed-address instructions (EXTEND, INHINT, RELINT, GO, RESUME)

### Encoder (`include/encode.h`, `src/encode.c`)

**Functions**:
- `moond_encode_result moond_encode_instr(const moond_instr* instr)`: Full encoding
- `moond_encode_result moond_encode_simple(moond_instr_type type, uint16_t address)`: Simplified API
- `bool moond_validate_address(moond_instr_type type, uint16_t address)`: Address range validation
- `uint16_t moond_max_address(moond_instr_type type)`: Get maximum valid address

**Encoding Table**:
Maps each instruction type to:
- Order code (opcode value)
- Opcode bit width (3 or 6 bits)
- Quarter code (if applicable)
- Address bit width (9 or 12 bits)
- EXTEND requirement flag

**Key Features**:
- Automatic address range validation
- Special handling for fixed-address instructions
- Returns success/error status with descriptive error messages
- Enforces correct address field sizes per instruction type

### Parity Handling

**Important**: Both encoder and decoder work with 15-bit words (AGC bits 1-15).  
Parity bit (AGC bit 0) is:
- **Ignored** during decoding
- **Synthesized** later during final assembly/recompilation

This is documented in all relevant headers and source files.

## Verified Instructions

### Working Correctly (Round-Trip):
- ✅ TC, CA, CS, AD, MSK (basic K-type, 12-bit address)
- ✅ MP (extracode K-type, 9-bit address, requires EXTEND)
- ✅ READ, WRITE, RAND, ROR (channel, 9-bit address)
- ✅ WAND, WOR, RXOR (extracode channel, requires EXTEND)
- ✅ EXTEND, INHINT, RELINT, GO (special fixed addresses)

### Address Range Enforcement:
- 12-bit instructions: 0-07777 octal (0-4095 decimal)
- 9-bit instructions (MP, channels): 0-00777 octal (0-511 decimal)

## Usage Examples

### Encoding
```c
// Simple encoding
moond_encode_result result = moond_encode_simple(INSTR_CA, OCTAL(00050));
if (result.success) {
    printf("Encoded: %05o\n", result.word);  // Prints: 30050
}

// With validation
if (moond_validate_address(INSTR_MP, OCTAL(01000))) {
    // Will fail - address too large for 9-bit field
}
```

### Decoding
```c
uint16_t word = OCTAL(30050);
moond_decoded_instr instr = moond_decode_instr(word, false);
printf("%s %s %05o\n",
       moond_instr_mnemonic(instr.type),
       moond_addr_mode_str(instr.addr_mode),
       instr.address);
// Prints: CA K 00050
```

### Round-Trip
```c
// Encode
moond_encode_result enc = moond_encode_simple(INSTR_MP, OCTAL(00234));

// Decode back (with EXTEND bit since MP requires it)
moond_decoded_instr dec = moond_decode_instr(enc.word, true);

// Verify
assert(dec.type == INSTR_MP);
assert(dec.address == OCTAL(00234));
assert(dec.requires_extend == true);
```

## Files

### Headers
- `include/instr.h` - Shared types and structures
- `include/encode.h` - Encoder API
- `include/decode.h` - Decoder API

### Implementation
- `src/instr.c` - Shared utility functions
- `src/encode.c` - Encoder implementation
- `src/decode.c` - Decoder implementation

### Tests
- `src/encode_test.c` - Encoder test with round-trip verification
- `src/decode_test.c` - Decoder test with known values

### Build
- `CMakeLists.txt` - Updated to build both encoder and decoder

## Future Work

- Complete quarter-code encoding for all E-type instructions (CCS, TS, XCH, etc.)
- Add support for RESUME (05.0017) special case
- Implement parity bit synthesis function
- Add disassembler output formatting
- Create assembler that accepts mnemonic input

## References

- `ref/block2/OPCODE_ENCODING.md` - Complete encoding specification
- `ref/moon/agcis_32_blk2_instructions.pdf` - Source PDF (pages 93-106 verified)
- `ref/block2/decoder_design.md` - Bit layout documentation
