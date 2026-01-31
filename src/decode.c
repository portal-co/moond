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
        .quarter_code = 0xFF,
        .status = -1
    };
    
    // Extract fields for all three formats
    uint8_t opcode_3 = moond_extract_opcode_3(word);     // Whole codes: bits 1-3
    uint8_t opcode_5 = moond_extract_opcode_5(word);     // Quarter codes: bits 1-5
    uint8_t opcode_6 = moond_extract_opcode_6(word);     // Channel: bits 1-6
    uint8_t quarter = moond_extract_quarter(word);       // Quarter codes: bits 6-8
    uint16_t addr_12 = moond_extract_addr_12(word);      // Whole codes: bits 4-15
    uint16_t addr_9 = moond_extract_addr_9(word);        // Channel: bits 7-15
    uint16_t addr_7 = moond_extract_addr_7(word);        // Quarter codes: bits 9-15
    
    result.opcode = opcode_3;  // Default to 3-bit for reporting
    
    // ========================================================================
    // SECTION 1: Channel Instructions (6-bit opcode 010 + 9-bit channel address)
    // These must be checked first as they use the longest opcode
    // ALL channel instructions (10.x) require EXTEND
    // ========================================================================
    
    if (opcode_6 == 010 && extend_bit) {
        result.opcode = opcode_6;
        // Use top 3 bits of channel address to distinguish variants
        uint8_t variant = (addr_9 >> 6) & 07;
        
        if (variant == 0) {
            result.type = INSTR_READ;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        } else if (variant == 1) {
            result.type = INSTR_WRITE;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        } else if (variant == 2) {
            result.type = INSTR_RAND;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        } else if (variant == 3) {
            result.type = INSTR_WAND;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        } else if (variant == 4) {
            result.type = INSTR_ROR;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        } else if (variant == 5) {
            result.type = INSTR_WOR;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        } else if (variant == 6) {
            result.type = INSTR_RXOR;
            result.addr_mode = ADDR_H;
            result.address = addr_9;
            result.requires_extend = true;
            result.status++;
        }
    }
    
    // ========================================================================
    // SECTION 2: Quarter Code Instructions (5-bit opcode + 3-bit quarter + 7-bit address)
    // ========================================================================
    
    // Order code 01.x
    if (opcode_5 == 01) {
        result.quarter_code = quarter;
        result.opcode = opcode_5;
        if (quarter == 0 && extend_bit) {
            result.type = INSTR_CCS;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        } else if (quarter == 2 || quarter == 4 || quarter == 6) {
            result.type = INSTR_TCF;
            result.addr_mode = ADDR_F;
            result.address = addr_7;
            result.status++;
        }
    }
    
    // Order code 02.x
    if (opcode_5 == 02) {
        result.quarter_code = quarter;
        result.opcode = opcode_5;
        if (extend_bit) {
            if (quarter == 0) {
                result.type = INSTR_DAS;
                result.addr_mode = ADDR_E;
                result.address = addr_7;
                result.requires_extend = true;
                result.status++;
            } else if (quarter == 2) {
                result.type = INSTR_LXCH;
                result.addr_mode = ADDR_E;
                result.address = addr_7;
                result.requires_extend = true;
                result.status++;
            } else if (quarter == 4) {
                result.type = INSTR_INCR;
                result.addr_mode = ADDR_E;
                result.address = addr_7;
                result.requires_extend = true;
                result.status++;
            } else if (quarter == 6) {
                result.type = INSTR_ADS;
                result.addr_mode = ADDR_E;
                result.address = addr_7;
                result.requires_extend = true;
                result.status++;
            }
        }
    }
    
    // Order code 05.x
    if (opcode_5 == 05) {
        result.quarter_code = quarter;
        result.opcode = opcode_5;
        if (quarter == 0) {
            if (extend_bit) {
                result.type = INSTR_NDX;
                result.addr_mode = ADDR_E;
                result.address = addr_7;
                result.requires_extend = true;
                result.status++;
            } else if (addr_7 == 017) {
                result.type = INSTR_RESUME;
                result.addr_mode = ADDR_NONE;
                result.status++;
            }
        } else if (quarter == 2 && extend_bit) {
            result.type = INSTR_DXCH;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        } else if (quarter == 4 && extend_bit) {
            result.type = INSTR_TS;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        } else if (quarter == 5 && extend_bit) {
            result.type = INSTR_XCH;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        }
    }
    
    // Order code 011.x (order code starts with 1, requires EXTEND)
    if (opcode_5 == 011 && extend_bit) {
        result.quarter_code = quarter;
        result.opcode = opcode_5;
        if (quarter == 0) {
            result.type = INSTR_DV;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        }
    }
    
    // Order code 012.x (order code starts with 1, ALL variants require EXTEND)
    if (opcode_5 == 012 && extend_bit) {
        result.quarter_code = quarter;
        result.opcode = opcode_5;
        result.requires_extend = true;
        
        if (quarter == 0) {
            result.type = INSTR_MSU;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.status++;
        } else if (quarter == 2) {
            result.type = INSTR_QXCH;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.status++;
        } else if (quarter == 4) {
            result.type = INSTR_AUG;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.status++;
        } else if (quarter == 6) {
            result.type = INSTR_DIM;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.status++;
        }
        // Note: 012.2, 012.4, 012.6 decode as BZMF when different condition?
        // Actually looking at the table, BZMF shares the same quarters as QXCH/AUG/DIM
        // The distinction must be extracode context - let me check the actual behavior needed
    }
    
    // Order code 013 - DCA (requires EXTEND, order code starts with 1)
    if (opcode_5 == 013 && extend_bit) {
        result.opcode = opcode_5;
        result.type = INSTR_DCA;
        result.addr_mode = ADDR_K;
        result.address = addr_7;
        result.requires_extend = true;
        result.status++;
    }
    
    // Order code 014 - DCS (requires EXTEND, order code starts with 1)
    if (opcode_5 == 014 && extend_bit) {
        result.opcode = opcode_5;
        result.type = INSTR_DCS;
        result.addr_mode = ADDR_K;
        result.address = addr_7;
        result.requires_extend = true;
        result.status++;
    }
    
    // Order code 015 - NDX K (requires EXTEND, order code starts with 1)
    if (opcode_5 == 015 && extend_bit) {
        result.opcode = opcode_5;
        result.type = INSTR_NDX;
        result.addr_mode = ADDR_K;
        result.address = addr_7;
        result.requires_extend = true;
        result.status++;
    }
    
    // Order code 016.x (order code starts with 1, so all variants need EXTEND checks)
    if (opcode_5 == 016) {
        result.quarter_code = quarter;
        result.opcode = opcode_5;
        if (quarter == 0 && extend_bit) {
            result.type = INSTR_SU;
            result.addr_mode = ADDR_E;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        } else if ((quarter == 2 || quarter == 4 || quarter == 6) && extend_bit) {
            result.type = INSTR_BZF;
            result.addr_mode = ADDR_F;
            result.address = addr_7;
            result.requires_extend = true;
            result.status++;
        }
    }
    
    // Order code 017 - MP (requires EXTEND, order code starts with 1)
    if (opcode_5 == 017 && extend_bit) {
        result.opcode = opcode_5;
        result.type = INSTR_MP;
        result.addr_mode = ADDR_K;
        result.address = addr_7;
        result.requires_extend = true;
        result.status++;
    }
    
    // ========================================================================
    // SECTION 3: Whole Code Instructions (3-bit opcode + 12-bit address)
    // These are checked last as they have the shortest opcode
    // ========================================================================
    
    // Order code 0. - TC and special addresses
    if (opcode_3 == 0) {
        if (addr_12 == 00006) {
            result.type = INSTR_EXTEND;
            result.addr_mode = ADDR_NONE;
            result.status++;
        } else if (addr_12 == 00004) {
            result.type = INSTR_INHINT;
            result.addr_mode = ADDR_NONE;
            result.status++;
        } else if (addr_12 == 00003) {
            result.type = INSTR_RELINT;
            result.addr_mode = ADDR_NONE;
            result.status++;
        } else if (addr_12 == 04000) {
            result.type = INSTR_GO;
            result.addr_mode = ADDR_NONE;
            result.status++;
        } else {
            result.type = INSTR_TC;
            result.addr_mode = ADDR_K;
            result.address = addr_12;
            result.status++;
        }
    }
    
    // Order code 3. - CA
    if (opcode_3 == 3) {
        result.type = INSTR_CA;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        result.status++;
    }
    
    // Order code 4. - CS
    if (opcode_3 == 4) {
        result.type = INSTR_CS;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        result.status++;
    }
    
    // Order code 6. - AD
    if (opcode_3 == 6) {
        result.type = INSTR_AD;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        result.status++;
    }
    
    // Order code 7. - MSK
    if (opcode_3 == 7) {
        result.type = INSTR_MSK;
        result.addr_mode = ADDR_K;
        result.address = addr_12;
        result.status++;
    }
    
    return result;
}
