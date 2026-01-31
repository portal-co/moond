// Test program for AGC Block-2 instruction encoder
// Demonstrates encoding various instruction types

#include "encode.h"
#include "decode.h"
#include "instr.h"
#include "core.h"
#include <stdio.h>

static void test_encode_decode(moond_instr_type type, uint16_t address, const char* expected) {
    // Encode
    moond_encode_result enc_result = moond_encode_simple(type, address);
    
    if (!enc_result.success) {
        printf("ENCODE FAILED: %s %05o - Error: %s\n",
               moond_instr_mnemonic(type), address, enc_result.error);
        return;
    }
    
    // Decode back
    bool needs_extend = moond_instr_needs_extend(type);
    moond_decoded_instr dec_result = moond_decode_instr(enc_result.word, needs_extend);
    
    // Verify round-trip
    bool match = (dec_result.type == type);
    const char* status = match ? "✓" : "✗";
    
    printf("%s %s %05o -> word=%05o -> %s %s %05o | Expected: %s\n",
           status,
           moond_instr_mnemonic(type),
           address,
           enc_result.word,
           moond_instr_mnemonic(dec_result.type),
           moond_addr_mode_str(dec_result.addr_mode),
           dec_result.address,
           expected);
}

int main(void) {
    printf("AGC Block-2 Instruction Encoder Test\n");
    printf("=====================================\n\n");
    
    printf("Note: Parity bits are synthesized later; encoder produces 15-bit words\n\n");
    
    // Basic instructions
    printf("Basic Instructions:\n");
    test_encode_decode(INSTR_TC, OCTAL(00000), "TC K 00000");
    test_encode_decode(INSTR_TC, OCTAL(00100), "TC K 00100");
    test_encode_decode(INSTR_CA, OCTAL(00050), "CA K 00050");
    test_encode_decode(INSTR_CS, OCTAL(00123), "CS K 00123");
    test_encode_decode(INSTR_AD, OCTAL(00100), "AD K 00100");
    test_encode_decode(INSTR_MSK, OCTAL(00777), "MSK K 00777");
    printf("\n");
    
    // Special instructions (fixed addresses)
    printf("Special Instructions:\n");
    test_encode_decode(INSTR_EXTEND, 0, "EXTEND");
    test_encode_decode(INSTR_INHINT, 0, "INHINT");
    test_encode_decode(INSTR_RELINT, 0, "RELINT");
    test_encode_decode(INSTR_RESUME, 0, "RESUME");
    test_encode_decode(INSTR_GO, 0, "GO");
    printf("\n");
    
    // Channel instructions
    printf("Channel Instructions:\n");
    test_encode_decode(INSTR_READ, OCTAL(030), "READ H 030");
    test_encode_decode(INSTR_WRITE, OCTAL(030), "WRITE H 030");
    test_encode_decode(INSTR_RAND, OCTAL(030), "RAND H 030");
    test_encode_decode(INSTR_ROR, OCTAL(030), "ROR H 030");
    printf("\n");
    
    // Extracode instructions
    printf("Extracode Instructions (require EXTEND prefix):\n");
    test_encode_decode(INSTR_MP, OCTAL(0234), "MP K 0234 (extracode)");
    test_encode_decode(INSTR_CCS, OCTAL(04050), "CCS E 04050 (extracode)");
    test_encode_decode(INSTR_TS, OCTAL(06050), "TS E 06050 (extracode)");
    test_encode_decode(INSTR_XCH, OCTAL(06450), "XCH E 06450 (extracode)");
    test_encode_decode(INSTR_WAND, OCTAL(030), "WAND H 030 (extracode)");
    test_encode_decode(INSTR_WOR, OCTAL(030), "WOR H 030 (extracode)");
    test_encode_decode(INSTR_RXOR, OCTAL(030), "RXOR H 030 (extracode)");
    printf("\n");
    
    // Test address range validation
    printf("Address Range Validation:\n");
    printf("Max address for TC (12-bit): %05o\n", moond_max_address(INSTR_TC));
    printf("Max address for MP (9-bit): %05o\n", moond_max_address(INSTR_MP));
    printf("Max address for READ (9-bit): %05o\n", moond_max_address(INSTR_READ));
    printf("\n");
    
    // Test invalid address
    printf("Testing out-of-range address:\n");
    moond_encode_result bad = moond_encode_simple(INSTR_MP, OCTAL(01000));  // Too large for 9-bit
    if (!bad.success) {
        printf("✓ Correctly rejected MP with address 01000: %s\n", bad.error);
    }
    printf("\n");
    
    printf("Test complete!\n");
    return 0;
}
