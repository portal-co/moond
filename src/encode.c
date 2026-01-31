#include "encode.h"
#include "bits.h"
#include "core.h"
#include <stddef.h>

// Encoding table: instruction type -> order code and address bits
typedef struct {
    moond_instr_type type;
    uint8_t opcode;           // Order code (3 or 6 bits)
    uint8_t opcode_bits;      // Number of bits in opcode (3 or 6)
    uint8_t quarter_code;     // Quarter code (0-7), or 0xff if not a quarter code
    uint8_t addr_bits;        // Number of address bits available
    bool requires_extend;     // Requires EXTEND prefix
} encode_entry;

static const encode_entry encode_table[] = {
    // Sequence changing
    {INSTR_TC,      OCTAL(00), 3, 0xff, 12, false},  // TC K - 00.
    {INSTR_TCF,     OCTAL(01), 6, 2,    12, false},  // TCF F - 01.2/4/6
    {INSTR_CCS,     OCTAL(01), 6, 0,    12, true},   // CCS E - 01.0 (extracode)
    {INSTR_BZF,     OCTAL(016), 6, 2,   12, false},  // BZF F - 16.2/4/6
    {INSTR_BZMF,    OCTAL(012), 6, 2,   12, false},  // BZMF F - 12.2/4/6
    
    // Fetching and storing
    {INSTR_CA,      OCTAL(03), 3, 0xff, 12, false},  // CA K - 03.
    {INSTR_CS,      OCTAL(04), 3, 0xff, 12, false},  // CS K - 04.
    {INSTR_DCA,     OCTAL(013), 6, 0xff, 9, false},  // DCA K - 13.
    {INSTR_DCS,     OCTAL(014), 6, 0xff, 9, false},  // DCS K - 14.
    {INSTR_TS,      OCTAL(05), 6, 4,    12, true},   // TS E - 05.4 (extracode)
    {INSTR_XCH,     OCTAL(05), 6, 5,    12, true},   // XCH E - 05.5 (extracode)
    {INSTR_LXCH,    OCTAL(02), 6, 2,    12, true},   // LXCH E - 02.2 (extracode)
    {INSTR_QXCH,    OCTAL(012), 6, 2,   12, true},   // QXCH E - 12.2 (extracode)
    {INSTR_DXCH,    OCTAL(05), 6, 2,    12, true},   // DXCH E - 05.2 (extracode)
    
    // Modifying
    {INSTR_NDX,     OCTAL(05), 6, 0,    12, false},  // NDX E/K - 05.0 or 15.
    
    // Arithmetic and logic
    {INSTR_AD,      OCTAL(06), 3, 0xff, 12, false},  // AD K - 06.
    {INSTR_SU,      OCTAL(016), 6, 0,   12, true},   // SU E - 16.0 (extracode)
    {INSTR_MP,      OCTAL(017), 6, 0xff, 9, true},   // MP K - 17. (extracode)
    {INSTR_DV,      OCTAL(011), 6, 0,   12, true},   // DV E - 11.0 (extracode)
    {INSTR_ADS,     OCTAL(02), 6, 6,    12, true},   // ADS E - 02.6 (extracode)
    {INSTR_DAS,     OCTAL(02), 6, 0,    12, true},   // DAS E - 02.0 (extracode)
    {INSTR_INCR,    OCTAL(02), 6, 4,    12, true},   // INCR E - 02.4 (extracode)
    {INSTR_AUG,     OCTAL(012), 6, 4,   12, true},   // AUG E - 12.4 (extracode)
    {INSTR_DIM,     OCTAL(012), 6, 6,   12, true},   // DIM E - 12.6 (extracode)
    {INSTR_MSU,     OCTAL(012), 6, 0,   12, true},   // MSU E - 12.0 (extracode)
    {INSTR_MSK,     OCTAL(07), 3, 0xff, 12, false},  // MSK K - 07.
    
    // Channel
    {INSTR_READ,    OCTAL(010), 6, 0,   9, false},   // READ H - 10.0
    {INSTR_WRITE,   OCTAL(010), 6, 1,   9, false},   // WRITE H - 10.1
    {INSTR_RAND,    OCTAL(010), 6, 2,   9, false},   // RAND H - 10.2
    {INSTR_WAND,    OCTAL(010), 6, 3,   9, true},    // WAND H - 10.3 (extracode)
    {INSTR_ROR,     OCTAL(010), 6, 4,   9, false},   // ROR H - 10.4
    {INSTR_WOR,     OCTAL(010), 6, 5,   9, true},    // WOR H - 10.5 (extracode)
    {INSTR_RXOR,    OCTAL(010), 6, 6,   9, true},    // RXOR H - 10.6 (extracode)
    
    // Special (fixed addresses)
    {INSTR_EXTEND,  OCTAL(00), 3, 0xff, 12, false},  // EXTEND - 00.0006
    {INSTR_INHINT,  OCTAL(00), 3, 0xff, 12, false},  // INHINT - 00.0004
    {INSTR_RELINT,  OCTAL(00), 3, 0xff, 12, false},  // RELINT - 00.0003
    {INSTR_RESUME,  OCTAL(05), 6, 0,    12, false},  // RESUME - 05.0017
    {INSTR_GO,      OCTAL(00), 3, 0xff, 12, false},  // GO - 00.4000
};

static const size_t encode_table_size = sizeof(encode_table) / sizeof(encode_table[0]);

static const encode_entry* find_encode_entry(moond_instr_type type) {
    for (size_t i = 0; i < encode_table_size; i++) {
        if (encode_table[i].type == type) {
            return &encode_table[i];
        }
    }
    return NULL;
}

bool moond_validate_address(moond_instr_type type, uint16_t address) {
    const encode_entry* entry = find_encode_entry(type);
    if (!entry) return false;
    
    uint16_t max_addr = (1 << entry->addr_bits) - 1;
    return address <= max_addr;
}

uint16_t moond_max_address(moond_instr_type type) {
    const encode_entry* entry = find_encode_entry(type);
    if (!entry) return 0;
    
    return (1 << entry->addr_bits) - 1;
}

moond_encode_result moond_encode_instr(const moond_instr* instr) {
    moond_encode_result result = {.success = false, .word = 0, .error = NULL};
    
    if (!instr) {
        result.error = "NULL instruction pointer";
        return result;
    }
    
    const encode_entry* entry = find_encode_entry(instr->type);
    if (!entry) {
        result.error = "Unknown instruction type";
        return result;
    }
    
    // Special case handling for fixed-address instructions
    uint16_t address = instr->address;
    
    switch (instr->type) {
        case INSTR_EXTEND:
            address = OCTAL(00006);
            break;
        case INSTR_INHINT:
            address = OCTAL(00004);
            break;
        case INSTR_RELINT:
            address = OCTAL(00003);
            break;
        case INSTR_RESUME:
            address = OCTAL(00017);
            break;
        case INSTR_GO:
            address = OCTAL(04000);
            break;
        default:
            // Validate address range
            if (!moond_validate_address(instr->type, address)) {
                result.error = "Address out of range for instruction type";
                return result;
            }
            break;
    }
    
    // Encode based on opcode size and quarter code
    // Note: Using bit reversal to properly encode AGC numeric values
    if (entry->opcode_bits == 3) {
        // 3-bit opcode: AGC bits 1-3 = opcode, bits 4-15 = address (12 bits)
        result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 3);
        result.word = insert_agc_bits_reversed(result.word, address & 0x0FFF, 4, 15);
    } else if (entry->quarter_code == 0xff) {
        // 6-bit whole code: AGC bits 1-6 = opcode, bits 7-15 = address (9 bits)
        result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 6);
        result.word = insert_agc_bits_reversed(result.word, address & 0x01FF, 7, 15);
    } else {
        // 6-bit quarter code: AGC bits 1-6 = opcode, bits 7-9 = quarter, bits 10-15 = address (6 bits)
        result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 6);
        result.word = insert_agc_bits_reversed(result.word, entry->quarter_code, 7, 9);
        result.word = insert_agc_bits_reversed(result.word, address & 0x3F, 10, 15);
    }
    
    result.success = true;
    return result;
}

moond_encode_result moond_encode_simple(moond_instr_type type, uint16_t address) {
    moond_instr instr = {
        .type = type,
        .address = address,
        .addr_mode = ADDR_K,  // Default, will be determined by encoder
        .requires_extend = moond_instr_needs_extend(type),
        .is_extracode = false,
        .opcode = 0,
        .quarter_code = 0xff
    };
    
    return moond_encode_instr(&instr);
}
