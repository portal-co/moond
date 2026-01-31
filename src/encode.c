#include "encode.h"
#include "bits.h"
#include "core.h"
#include <stddef.h>

// Encoding table: instruction type -> order code and address bits
typedef struct {
    moond_instr_type type;
    uint8_t opcode;           // Order code value
    uint8_t opcode_bits;      // Number of bits in opcode (3, 5, or 6)
    uint8_t quarter_code;     // Quarter code (0-7), or 0xff if not a quarter code
    uint8_t addr_bits;        // Number of address bits available
    bool requires_extend;     // Requires EXTEND prefix
} encode_entry;

static const encode_entry encode_table[] = {
    // Whole code instructions (3-bit opcode + 12-bit address)
    // All have implicit leading 0, so never require EXTEND
    {INSTR_TC,      0, 3, 0xff, 12, false},     // TC K - 0.
    {INSTR_CA,      3, 3, 0xff, 12, false},     // CA K - 3.
    {INSTR_CS,      4, 3, 0xff, 12, false},     // CS K - 4.
    {INSTR_AD,      6, 3, 0xff, 12, false},     // AD K - 6.
    {INSTR_MSK,     7, 3, 0xff, 12, false},     // MSK K - 7.
    {INSTR_EXTEND,  0, 3, 0xff, 12, false},     // EXTEND - 0.0006
    {INSTR_INHINT,  0, 3, 0xff, 12, false},     // INHINT - 0.0004
    {INSTR_RELINT,  0, 3, 0xff, 12, false},     // RELINT - 0.0003
    {INSTR_GO,      0, 3, 0xff, 12, false},     // GO - 0.4000
    
    // Quarter code instructions starting with 0 (5-bit opcode + 3-bit quarter + 7-bit address)
    {INSTR_TCF,     01, 5, 2,    7, false},     // TCF F - 01.2/4/6 (no EXTEND)
    {INSTR_CCS,     01, 5, 0,    7, true},      // CCS E - 01.0 (requires EXTEND)
    {INSTR_DAS,     02, 5, 0,    7, true},      // DAS E - 02.0 (requires EXTEND)
    {INSTR_LXCH,    02, 5, 2,    7, true},      // LXCH E - 02.2 (requires EXTEND)
    {INSTR_INCR,    02, 5, 4,    7, true},      // INCR E - 02.4 (requires EXTEND)
    {INSTR_ADS,     02, 5, 6,    7, true},      // ADS E - 02.6 (requires EXTEND)
    {INSTR_NDX,     05, 5, 0,    7, true},      // NDX E - 05.0 (requires EXTEND for E variant)
    {INSTR_DXCH,    05, 5, 2,    7, true},      // DXCH E - 05.2 (requires EXTEND)
    {INSTR_TS,      05, 5, 4,    7, true},      // TS E - 05.4 (requires EXTEND)
    {INSTR_XCH,     05, 5, 5,    7, true},      // XCH E - 05.5 (requires EXTEND)
    {INSTR_RESUME,  05, 5, 0,    7, false},     // RESUME - 05.0017 (no EXTEND)
    
    // Quarter code instructions starting with 1 (ALL require EXTEND)
    {INSTR_DV,      011, 5, 0,   7, true},      // DV E - 011.0 (requires EXTEND)
    {INSTR_MSU,     012, 5, 0,   7, true},      // MSU E - 012.0 (requires EXTEND)
    {INSTR_QXCH,    012, 5, 2,   7, true},      // QXCH E - 012.2 (requires EXTEND)
    {INSTR_AUG,     012, 5, 4,   7, true},      // AUG E - 012.4 (requires EXTEND)
    {INSTR_DIM,     012, 5, 6,   7, true},      // DIM E - 012.6 (requires EXTEND)
    {INSTR_BZMF,    012, 5, 2,   7, true},      // BZMF F - 012.2/4/6 (requires EXTEND per order code rule)
    {INSTR_DCA,     013, 5, 0xff, 7, true},     // DCA K - 013. (requires EXTEND)
    {INSTR_DCS,     014, 5, 0xff, 7, true},     // DCS K - 014. (requires EXTEND)
    {INSTR_SU,      016, 5, 0,   7, true},      // SU E - 016.0 (requires EXTEND)
    {INSTR_BZF,     016, 5, 2,   7, true},      // BZF F - 016.2/4/6 (requires EXTEND)
    {INSTR_MP,      017, 5, 0xff, 7, true},     // MP K - 017. (requires EXTEND)
    
    // Channel instructions (6-bit opcode + 9-bit channel address)
    // ALL channel instructions (10.x) require EXTEND
    {INSTR_READ,    010, 6, 0xff, 9, true},     // READ H - 10.0 (requires EXTEND)
    {INSTR_WRITE,   010, 6, 0xff, 9, true},     // WRITE H - 10.1 (requires EXTEND)
    {INSTR_RAND,    010, 6, 0xff, 9, true},     // RAND H - 10.2 (requires EXTEND)
    {INSTR_WAND,    010, 6, 0xff, 9, true},     // WAND H - 10.3 (requires EXTEND)
    {INSTR_ROR,     010, 6, 0xff, 9, true},     // ROR H - 10.4 (requires EXTEND)
    {INSTR_WOR,     010, 6, 0xff, 9, true},     // WOR H - 10.5 (requires EXTEND)
    {INSTR_RXOR,    010, 6, 0xff, 9, true},     // RXOR H - 10.6 (requires EXTEND)
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
        // 3-bit whole code: AGC bits 1-3 = opcode, bits 4-15 = address (12 bits)
        result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 3);
        result.word = insert_agc_bits_reversed(result.word, address & 0x0FFF, 4, 15);
    } else if (entry->opcode_bits == 5) {
        // 5-bit quarter code: AGC bits 1-5 = opcode, bits 6-8 = quarter, bits 9-15 = address (7 bits)
        result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 5);
        if (entry->quarter_code != 0xff) {
            result.word = insert_agc_bits_reversed(result.word, entry->quarter_code, 6, 8);
        }
        result.word = insert_agc_bits_reversed(result.word, address & 0x7F, 9, 15);
    } else if (entry->opcode_bits == 6) {
        // 6-bit channel: AGC bits 1-6 = opcode, bits 7-15 = channel address (9 bits)
        result.word = insert_agc_bits_reversed(0, entry->opcode, 1, 6);
        result.word = insert_agc_bits_reversed(result.word, address & 0x01FF, 7, 15);
    } else {
        result.error = "Invalid opcode bit width";
        return result;
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
