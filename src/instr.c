#include "instr.h"
#include "core.h"
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
        case INSTR_BZF:
        case INSTR_BZMF:
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
        // Channel extracode instructions (all 10.x variants require EXTEND)
        case INSTR_READ:
        case INSTR_WRITE:
        case INSTR_RAND:
        case INSTR_WAND:
        case INSTR_ROR:
        case INSTR_WOR:
        case INSTR_RXOR:
        // K-type extracode
        case INSTR_DCA:
        case INSTR_DCS:
        case INSTR_MP:
            return true;
        // NDX can be either E (extracode) or K (basic)
        case INSTR_NDX:
            return false;  // Depends on context
        default:
            return false;
    }
}
