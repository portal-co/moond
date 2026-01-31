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
    
    // Extract all possible opcode and address fields
    uint8_t opcode_3 = moond_extract_opcode_3(word);
    uint8_t opcode_6 = moond_extract_opcode_6(word);
    uint8_t quarter = moond_extract_quarter(word);
    uint16_t addr_12 = moond_extract_addr_12(word);
    uint16_t addr_9 = moond_extract_addr_9(word);
    uint16_t addr_6 = moond_extract_addr_6(word);
    
    result.opcode = opcode_3;  // Default to 3-bit opcode
    
    // ========================================================================
    // SECTION 1: 6-bit Quarter Code Instructions (check FIRST)
    // Format: opcode (6 bits) + quarter (3 bits) + address (6 bits)
    // ========================================================================
    
    // Order code 01.x
    if (opcode_6 == 01) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // CCS E - 01.0 (extracode)
            result.type = INSTR_CCS;
            result.addr_mode = ADDR_E;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2 || quarter == 4 || quarter == 6) {
            // TCF F - 01.2, 01.4, 01.6
            result.type = INSTR_TCF;
            result.addr_mode = ADDR_F;
            result.address = addr_6;
            return result;
        }
    }
    
    // Order code 02.x
    if (opcode_6 == 02) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (extend_bit) {
            if (quarter == 0) {
                // DAS E - 02.0 (extracode)
                result.type = INSTR_DAS;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (quarter == 2) {
                // LXCH E - 02.2 (extracode)
                result.type = INSTR_LXCH;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (quarter == 4) {
                // INCR E - 02.4 (extracode)
                result.type = INSTR_INCR;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (quarter == 6) {
                // ADS E - 02.6 (extracode)
                result.type = INSTR_ADS;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            }
        }
    }
    
    // Order code 05.x
    if (opcode_6 == 05) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0) {
            if (extend_bit) {
                // NDX E - 05.0 (extracode version)
                result.type = INSTR_NDX;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (addr_6 == 017) {
                // RESUME - 05.0017 (special)
                result.type = INSTR_RESUME;
                result.addr_mode = ADDR_NONE;
                return result;
            }
        } else if (quarter == 2 && extend_bit) {
            // DXCH E - 05.2 (extracode)
            result.type = INSTR_DXCH;
            result.addr_mode = ADDR_E;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        } else if (quarter == 4 && extend_bit) {
            // TS E - 05.4 (extracode)
            result.type = INSTR_TS;
            result.addr_mode = ADDR_E;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        } else if (quarter == 5 && extend_bit) {
            // XCH E - 05.5 (extracode)
            result.type = INSTR_XCH;
            result.addr_mode = ADDR_E;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        }
    }
    
    // Order code 010.x (channel instructions)
    if (opcode_6 == 010) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0) {
            // READ H - 010.0
            result.type = INSTR_READ;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            return result;
        } else if (quarter == 1) {
            // WRITE H - 010.1
            result.type = INSTR_WRITE;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            return result;
        } else if (quarter == 2) {
            // RAND H - 010.2
            result.type = INSTR_RAND;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            return result;
        } else if (quarter == 3 && extend_bit) {
            // WAND H - 010.3 (extracode)
            result.type = INSTR_WAND;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        } else if (quarter == 4) {
            // ROR H - 010.4
            result.type = INSTR_ROR;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            return result;
        } else if (quarter == 5 && extend_bit) {
            // WOR H - 010.5 (extracode)
            result.type = INSTR_WOR;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        } else if (quarter == 6 && extend_bit) {
            // RXOR H - 010.6 (extracode)
            result.type = INSTR_RXOR;
            result.addr_mode = ADDR_H;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        }
    }
    
    // Order code 011.x
    if (opcode_6 == 011) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // DV E - 011.0 (extracode)
            result.type = INSTR_DV;
            result.addr_mode = ADDR_E;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        }
    }
    
    // Order code 012.x
    if (opcode_6 == 012) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (extend_bit) {
            if (quarter == 0) {
                // MSU E - 012.0 (extracode)
                result.type = INSTR_MSU;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (quarter == 2) {
                // QXCH E - 012.2 (extracode)
                result.type = INSTR_QXCH;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (quarter == 4) {
                // AUG E - 012.4 (extracode)
                result.type = INSTR_AUG;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            } else if (quarter == 6) {
                // DIM E - 012.6 (extracode)
                result.type = INSTR_DIM;
                result.addr_mode = ADDR_E;
                result.address = addr_6;
                result.requires_extend = true;
                return result;
            }
        }
        // Also handles BZMF F and other non-extracode variants
        if (quarter == 2 || quarter == 4 || quarter == 6) {
            // BZMF F - 012.2, 012.4, 012.6
            result.type = INSTR_BZMF;
            result.addr_mode = ADDR_F;
            result.address = addr_6;
            return result;
        }
    }
    
    // ========================================================================
    // SECTION 2: 6-bit Whole Code Instructions (no quarter code)
    // Format: opcode (6 bits) + address (9 bits)
    // ========================================================================
    
    // Order code 013 - DCA
    if (opcode_6 == 013) {
        result.opcode = opcode_6;
        result.type = INSTR_DCA;
        result.addr_mode = ADDR_K;
        result.address = addr_9;
        return result;
    }
    
    // Order code 014 - DCS
    if (opcode_6 == 014) {
        result.opcode = opcode_6;
        result.type = INSTR_DCS;
        result.addr_mode = ADDR_K;
        result.address = addr_9;
        return result;
    }
    
    // Order code 015 - NDX (basic, non-extracode)
    if (opcode_6 == 015) {
        result.opcode = opcode_6;
        result.type = INSTR_NDX;
        result.addr_mode = ADDR_K;
        result.address = addr_9;
        return result;
    }
    
    // Order code 016.x
    if (opcode_6 == 016) {
        result.quarter_code = quarter;
        result.opcode = opcode_6;
        if (quarter == 0 && extend_bit) {
            // SU E - 016.0 (extracode)
            result.type = INSTR_SU;
            result.addr_mode = ADDR_E;
            result.address = addr_6;
            result.requires_extend = true;
            return result;
        } else if (quarter == 2 || quarter == 4 || quarter == 6) {
            // BZF F - 016.2, 016.4, 016.6
            result.type = INSTR_BZF;
            result.addr_mode = ADDR_F;
            result.address = addr_6;
            return result;
        }
    }
    
    // Order code 017 - MP (extracode)
    if (opcode_6 == 017) {
        result.opcode = opcode_6;
        result.type = INSTR_MP;
        result.addr_mode = ADDR_K;
        result.address = addr_9;
        result.requires_extend = true;
        return result;
    }
    
    // ========================================================================
    // SECTION 3: 3-bit Basic Instructions
    // Format: opcode (3 bits) + address (12 bits)
    // ========================================================================
    
    // Order code 00. - TC (and special fixed addresses)
    if (opcode_3 == 00) {
        if (addr_12 == 00006) {
            result.type = INSTR_EXTEND;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == 00004) {
            result.type = INSTR_INHINT;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == 00003) {
            result.type = INSTR_RELINT;
            result.addr_mode = ADDR_NONE;
            return result;
        } else if (addr_12 == 04000) {
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
    
    // Order code 03. - CA
    if (opcode_3 == 03) {
        result.type = INSTR_CA;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    // Order code 04. - CS
    if (opcode_3 == 04) {
        result.type = INSTR_CS;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    // Order code 06. - AD
    if (opcode_3 == 06) {
        result.type = INSTR_AD;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    // Order code 07. - MSK
    if (opcode_3 == 07) {
        result.type = INSTR_MSK;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        return result;
    }
    
    // If we get here, instruction is unknown
    return result;
}
