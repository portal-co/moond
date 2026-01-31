#include "decode.h"
#include "core.h"
#include <stddef.h>

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
    // Use OCTAL() macro for clarity on octal constants
    
    // Special fixed addresses (checked first)
    if (opcode_3 == OCTAL(00)) {
        if (addr_12 == OCTAL(00006)) {
            result.type = INSTR_EXTEND;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == OCTAL(00004)) {
            result.type = INSTR_INHINT;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == OCTAL(00003)) {
            result.type = INSTR_RELINT;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == OCTAL(04000)) {
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
    if (opcode_6 == OCTAL(01)) {
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
    
    if (opcode_6 == OCTAL(02)) {
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
    
    if (opcode_3 == OCTAL(03)) {
        // CA K - order code 03.
        result.type = INSTR_CA;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_3 == OCTAL(04)) {
        // CS K - order code 04.
        result.type = INSTR_CS;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == OCTAL(05)) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (addr_12 == OCTAL(00017)) {
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
    
    if (opcode_3 == OCTAL(06)) {
        // AD K - order code 06.
        result.type = INSTR_AD;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_3 == OCTAL(07)) {
        // MSK K - order code 07.
        result.type = INSTR_MSK;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == OCTAL(010)) {
        // 10.X - Channel instructions
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
    
    if (opcode_6 == OCTAL(011)) {
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
    
    if (opcode_6 == OCTAL(012)) {
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
    
    if (opcode_6 == OCTAL(013)) {
        // DCA K - order code 13. (6-bit whole code)
        result.opcode = opcode_6;
        result.type = INSTR_DCA;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == OCTAL(014)) {
        // DCS K - order code 14. (6-bit whole code)
        result.opcode = opcode_6;
        result.type = INSTR_DCS;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == OCTAL(015)) {
        // NDX K - order code 15.
        result.opcode = opcode_6;
        result.type = INSTR_NDX;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    if (opcode_6 == OCTAL(016)) {
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
    
    if (opcode_6 == OCTAL(017)) {
        // MP K - order code 17. (6-bit extracode, requires EXTEND)
        // Uses 9-bit address field (bits 7-15), not 12-bit
        result.opcode = opcode_6;
        result.type = INSTR_MP;
        result.addr_mode = ADDR_K;
        result.address = addr_9;  // 9-bit address, not 12-bit
        result.requires_extend = true;  // MP requires EXTEND prefix
        return result;
    }
    
    // If we reach here, instruction is unknown
    return result;
}
