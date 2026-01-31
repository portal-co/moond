#include "decode.h"
#include <stddef.h>

// Instruction mnemonic lookup table
static const char* instr_mnemonics[] = {
    [INSTR_TC] = "TC",
    [INSTR_TCF] = "TCF",
    [INSTR_CCS] = "CCS",
    [INSTR_BZF] = "BZF",
    [INSTR_BZMF] = "BZMF",
    [INSTR_CA] = "CA",
    [INSTR_CS] = "CS",
    [INSTR_DCA] = "DCA",
    [INSTR_DCS] = "DCS",
    [INSTR_TS] = "TS",
    [INSTR_XCH] = "XCH",
    [INSTR_LXCH] = "LXCH",
    [INSTR_QXCH] = "QXCH",
    [INSTR_DXCH] = "DXCH",
    [INSTR_NDX] = "NDX",
    [INSTR_AD] = "AD",
    [INSTR_SU] = "SU",
    [INSTR_MP] = "MP",
    [INSTR_DV] = "DV",
    [INSTR_ADS] = "ADS",
    [INSTR_DAS] = "DAS",
    [INSTR_INCR] = "INCR",
    [INSTR_AUG] = "AUG",
    [INSTR_DIM] = "DIM",
    [INSTR_MSU] = "MSU",
    [INSTR_MSK] = "MSK",
    [INSTR_READ] = "READ",
    [INSTR_WRITE] = "WRITE",
    [INSTR_RAND] = "RAND",
    [INSTR_WAND] = "WAND",
    [INSTR_ROR] = "ROR",
    [INSTR_WOR] = "WOR",
    [INSTR_RXOR] = "RXOR",
    [INSTR_EXTEND] = "EXTEND",
    [INSTR_INHINT] = "INHINT",
    [INSTR_RELINT] = "RELINT",
    [INSTR_RESUME] = "RESUME",
    [INSTR_GO] = "GO",
    [INSTR_PINC] = "PINC",
    [INSTR_MINC] = "MINC",
    [INSTR_DINC] = "DINC",
    [INSTR_PCDU] = "PCDU",
    [INSTR_MCDU] = "MCDU",
    [INSTR_SHINC] = "SHINC",
    [INSTR_SHANC] = "SHANC",
    [INSTR_TCSAJ] = "TCSAJ",
    [INSTR_FETCH] = "FETCH",
    [INSTR_STORE] = "STORE",
    [INSTR_INOTRD] = "INOTRD",
    [INSTR_INOTLD] = "INOTLD",
    [INSTR_UNKNOWN] = "UNKNOWN"
};

static const char* addr_mode_strs[] = {
    [ADDR_K] = "K",
    [ADDR_E] = "E",
    [ADDR_F] = "F",
    [ADDR_H] = "H",
    [ADDR_C] = "C",
    [ADDR_NONE] = "NONE"
};

const char* moond_instr_mnemonic(moond_instr_type type) {
    if (type >= 0 && type < sizeof(instr_mnemonics) / sizeof(instr_mnemonics[0])) {
        return instr_mnemonics[type];
    }
    return "INVALID";
}

const char* moond_addr_mode_str(moond_addr_mode mode) {
    if (mode >= 0 && mode < sizeof(addr_mode_strs) / sizeof(addr_mode_strs[0])) {
        return addr_mode_strs[mode];
    }
    return "INVALID";
}

bool moond_instr_needs_extend(moond_instr_type type) {
    switch (type) {
        // E-type extracode instructions
        case INSTR_CCS:
        case INSTR_TS:
        case INSTR_XCH:
        case INSTR_LXCH:
        case INSTR_QXCH:
        case INSTR_DXCH:
        case INSTR_SU:
        case INSTR_DV:
        case INSTR_ADS:
        case INSTR_DAS:
        case INSTR_INCR:
        case INSTR_AUG:
        case INSTR_DIM:
        case INSTR_MSU:
        case INSTR_STORE:
        // Channel extracode instructions
        case INSTR_WAND:
        case INSTR_WOR:
        case INSTR_RXOR:
            return true;
        // NDX can be either E (extracode) or K (basic)
        case INSTR_NDX:
            return false;  // Depends on context
        default:
            return false;
    }
}

moond_decoded_instr moond_decode_instr(uint16_t word, bool extend_bit) {
    moond_decoded_instr result = {
        .type = INSTR_UNKNOWN,
        .addr_mode = ADDR_NONE,
        .address = 0,
        .requires_extend = false,
        .is_extracode = extend_bit,
        .opcode = 0,
        .quarter_code = 0xFF
    };
    
    // Extract opcode fields
    uint8_t opcode_3 = moond_extract_opcode_3(word);  // AGC bits 1-3 (top 3 bits)
    uint8_t opcode_6 = moond_extract_opcode_6(word);  // AGC bits 1-6 (top 6 bits, for quarter codes)
    uint8_t quarter = moond_extract_quarter(word);     // AGC bits 7-9
    uint16_t addr_12 = moond_extract_addr_12(word);   // AGC bits 4-15
    uint16_t addr_9 = moond_extract_addr_9(word);     // AGC bits 7-15
    
    result.opcode = opcode_3;  // Store 3-bit opcode by default
    
    // Decode based on opcode and extend bit
    // Order codes are in octal format (from OPCODE_ENCODING.md)
    
    // Special fixed addresses (checked first)
    if (opcode_3 == 0) {  // 00. (octal) - TC or special
        if (addr_12 == 0006) {  // octal 00006
            result.type = INSTR_EXTEND;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == 0004) {  // octal 00004
            result.type = INSTR_INHINT;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == 0003) {  // octal 00003
            result.type = INSTR_RELINT;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == 02000) {  // octal 04000
            result.type = INSTR_GO;
            result.addr_mode = ADDR_NONE;
            return result;
        } else {
            // TC K - order code 00.
            result.type = INSTR_TC;
            result.addr_mode = ADDR_K;
            result.address = addr_12;
            return result;
        }
    }
    
    // Quarter codes - check 6-bit opcode
    if (opcode_6 == 01) {  // 01.X (octal)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // CCS E - order code 01.0 (extracode)
            result.type = INSTR_CCS;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2 || quarter == 4 || quarter == 6) {
            // TCF F - order code 01.2, 01.4, 01.6
            result.type = INSTR_TCF;
            result.addr_mode = ADDR_F;
            result.address = addr_12;
            return result;
        }
    }
    
    if (opcode_6 == 02) {  // 02.X (octal)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // DAS E - order code 02.0 (extracode)
            result.type = INSTR_DAS;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2) {
            // LXCH E - order code 02.2 (may need extracode)
            result.type = INSTR_LXCH;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = extend_bit;
            return result;
        } else if (quarter == 4 && extend_bit) {
            // INCR E - order code 02.4 (extracode)
            result.type = INSTR_INCR;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 6 && extend_bit) {
            // ADS E - order code 02.6 (extracode)
            result.type = INSTR_ADS;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        }
    }
    
    if (opcode_3 == 03) {  // 03. (octal)
        // CA K - order code 03.
        result.type = INSTR_CA;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_3 == 04) {  // 04. (octal)
        // CS K - order code 04.
        result.type = INSTR_CS;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == 05) {  // 05.X (octal)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (addr_12 == 0017) {  // octal 00017
            // RESUME - order code 05.0017
            result.type = INSTR_RESUME;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (quarter == 0) {
            // NDX E (extracode) or NDX K (basic) - order code 05.0
            result.type = INSTR_NDX;
            result.addr_mode = extend_bit ? ADDR_E : ADDR_K;
            result.address = addr_12;
            result.requires_extend = extend_bit;
            return result;
        } else if (quarter == 2 && extend_bit) {
            // DXCH E - order code 05.2 (extracode)
            result.type = INSTR_DXCH;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 4 && extend_bit) {
            // TS E - order code 05.4 (extracode)
            result.type = INSTR_TS;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 5) {
            // XCH E - order code 05.5 (may need extracode)
            result.type = INSTR_XCH;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = extend_bit;
            return result;
        }
    }
    
    if (opcode_3 == 06) {  // 06. (octal)
        // AD K - order code 06.
        result.type = INSTR_AD;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_3 == 07) {  // 07. (octal)
        // MSK K - order code 07.
        result.type = INSTR_MSK;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == 010) {  // 10.X (octal - channel instructions)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        result.address = addr_9;
        result.addr_mode = ADDR_H;
        
        switch (quarter) {
            case 0:
                result.type = INSTR_READ;
                return result;
            case 1:
                result.type = INSTR_WRITE;
                return result;
            case 2:
                result.type = INSTR_RAND;
                return result;
            case 3:
                if (extend_bit) {
                    result.type = INSTR_WAND;
                    result.requires_extend = true;
                    return result;
                }
                break;
            case 4:
                result.type = INSTR_ROR;
                return result;
            case 5:
                if (extend_bit) {
                    result.type = INSTR_WOR;
                    result.requires_extend = true;
                    return result;
                }
                break;
            case 6:
                if (extend_bit) {
                    result.type = INSTR_RXOR;
                    result.requires_extend = true;
                    return result;
                }
                break;
        }
    }
    
    if (opcode_6 == 011) {  // 11.X (octal)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // DV E - order code 11.0 (extracode)
            result.type = INSTR_DV;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        }
    }
    
    if (opcode_6 == 012) {  // 12.X (octal)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // MSU E - order code 12.0 (extracode)
            result.type = INSTR_MSU;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2) {
            // QXCH E - order code 12.2 (may need extracode)
            result.type = INSTR_QXCH;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = extend_bit;
            return result;
        } else if (quarter == 4 && extend_bit) {
            // AUG E - order code 12.4 (extracode)
            result.type = INSTR_AUG;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 6 && extend_bit) {
            // DIM E - order code 12.6 (extracode)
            result.type = INSTR_DIM;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2 || quarter == 4 || quarter == 6) {
            // BZMF F - order code 12.2, 12.4, 12.6 (basic mode)
            if (!extend_bit) {
                result.type = INSTR_BZMF;
                result.addr_mode = ADDR_F;
                result.address = addr_12;
                return result;
            }
        }
    }
    
    if (opcode_3 == 013) {  // 13. (octal - note: this is 11 decimal, so 0xB, but in C octal is 013)
        // DCA K - order code 13.
        result.type = INSTR_DCA;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_3 == 014) {  // 14. (octal)
        // DCS K - order code 14.
        result.type = INSTR_DCS;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == 015) {  // 15. (octal)
        // NDX K - order code 15.
        result.opcode = opcode_6;
        result.type = INSTR_NDX;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == 016) {  // 16.X (octal)
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // SU E - order code 16.0 (extracode)
            result.type = INSTR_SU;
            result.addr_mode = ADDR_E;
            result.address = addr_12;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2 || quarter == 4 || quarter == 6) {
            // BZF F - order code 16.2, 16.4, 16.6
            result.type = INSTR_BZF;
            result.addr_mode = ADDR_F;
            result.address = addr_12;
            return result;
        }
    }
    
    if (opcode_3 == 017) {  // 17. (octal)
        // MP K - order code 17.
        result.type = INSTR_MP;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    // If we reach here, instruction is unknown
    return result;
}
