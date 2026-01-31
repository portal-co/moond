// Test program for AGC Block-2 instruction decoder
// Demonstrates decoding various instruction types

#include "decode.h"
#include <stdio.h>

// Helper to convert octal string to 15-bit word
// e.g., "030050" -> 0x3028
static uint16_t octal_to_word(const char* octal_str) {
    uint16_t result = 0;
    while (*octal_str) {
        result = (result << 3) | (*octal_str++ - '0');
    }
    return result & 0x7FFF;  // 15 bits
}

static void test_decode(const char* octal_word, bool extend, const char* expected) {
    uint16_t word = octal_to_word(octal_word);
    moond_decoded_instr instr = moond_decode_instr(word, extend);
    
    printf("Word: %s (0x%04x, extend=%d) -> %s %s",
           octal_word, word, extend,
           moond_instr_mnemonic(instr.type),
           moond_addr_mode_str(instr.addr_mode));
    
    if (instr.addr_mode != ADDR_NONE) {
        if (instr.addr_mode == ADDR_H) {
            printf(" %04o", instr.address);
        } else {
            printf(" %05o", instr.address);
        }
    }
    
    if (instr.requires_extend) {
        printf(" [needs EXTEND]");
    }
    
    printf(" | Expected: %s", expected);
    printf("\n");
}

int main(void) {
    printf("AGC Block-2 Instruction Decoder Test\n");
    printf("=====================================\n\n");
    
    // Basic instructions (no EXTEND)
    printf("Basic Instructions:\n");
    test_decode("00000", false, "TC K 00000");
    test_decode("00100", false, "TC K 00100");
    test_decode("30050", false, "CA K 00050");
    test_decode("40123", false, "CS K 00123");
    test_decode("60100", false, "AD K 00100");
    test_decode("74234", false, "MP K 04234");
    test_decode("70777", false, "MSK K 00777");
    test_decode("54345", false, "DCA K 04345");
    test_decode("64456", false, "DCS K 04456");
    printf("\n");
    
    // Special instructions
    printf("Special Instructions:\n");
    test_decode("00006", false, "EXTEND");
    test_decode("00004", false, "INHINT");
    test_decode("00003", false, "RELINT");
    test_decode("24017", false, "RESUME");
    test_decode("04000", false, "GO");
    printf("\n");
    
    // Channel instructions
    printf("Channel Instructions:\n");
    test_decode("40030", false, "READ H 030");
    test_decode("40130", false, "WRITE H 030");
    test_decode("40230", false, "RAND H 030");
    test_decode("40430", false, "ROR H 030");
    printf("\n");
    
    // Extracode instructions (require EXTEND)
    printf("Extracode E-Type Instructions:\n");
    test_decode("04050", true, "CCS E 04050 (extracode)");
    test_decode("26050", true, "TS E 06050 (extracode)");
    test_decode("26450", true, "XCH E 06450 (extracode)");
    test_decode("10050", true, "LXCH E 00050 (extracode)");
    test_decode("50050", true, "QXCH E 00050 (extracode)");
    test_decode("25050", true, "DXCH E 05050 (extracode)");
    test_decode("70050", true, "SU E 00050 (extracode)");
    test_decode("44050", true, "DV E 04050 (extracode)");
    test_decode("13050", true, "ADS E 03050 (extracode)");
    test_decode("10050", true, "DAS E 00050 (extracode)");
    test_decode("11050", true, "INCR E 01050 (extracode)");
    test_decode("52050", true, "AUG E 02050 (extracode)");
    test_decode("53050", true, "DIM E 03050 (extracode)");
    test_decode("50050", true, "MSU E 00050 (extracode)");
    printf("\n");
    
    // Extracode channel instructions
    printf("Extracode Channel Instructions:\n");
    test_decode("40330", true, "WAND H 030 (extracode)");
    test_decode("40530", true, "WOR H 030 (extracode)");
    test_decode("40630", true, "RXOR H 030 (extracode)");
    printf("\n");
    
    // NDX - different modes
    printf("NDX Instruction (context-dependent):\n");
    test_decode("24100", false, "NDX K 04100 (basic)");
    test_decode("24100", true, "NDX E 04100 (extracode)");
    test_decode("64100", false, "NDX K 04100 (always K-type)");
    printf("\n");
    
    // TCF with different quarter codes
    printf("TCF Variations:\n");
    test_decode("05345", false, "TCF F 05345 (quarter 2)");
    test_decode("06345", false, "TCF F 06345 (quarter 4)");
    test_decode("07345", false, "TCF F 07345 (quarter 6)");
    printf("\n");
    
    // BZF and BZMF
    printf("Branch Instructions:\n");
    test_decode("71345", false, "BZF F 01345");
    test_decode("72345", false, "BZF F 02345");
    test_decode("73345", false, "BZF F 03345");
    test_decode("51345", false, "BZMF F 01345");
    test_decode("52345", false, "BZMF F 02345");
    test_decode("53345", false, "BZMF F 03345");
    printf("\n");
    
    // Demonstrate extracode mode changing interpretation
    printf("Extracode vs Basic Mode Disambiguation:\n");
    test_decode("04100", false, "BZF F (without EXTEND)");
    test_decode("04100", true, "CCS E (with EXTEND)");
    test_decode("70100", false, "BZF F (without EXTEND)");
    test_decode("70100", true, "SU E (with EXTEND)");
    printf("\n");
    
    printf("Test complete!\n");
    return 0;
}
