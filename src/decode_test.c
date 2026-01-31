// Test program for AGC Block-2 instruction encoder/decoder
// Tests encode-decode round-trip and collision detection

#include "decode.h"
#include "encode.h"
#include <stdio.h>
#include <stdbool.h>

static int test_count = 0;
static int pass_count = 0;
static int fail_count = 0;

// Test encode-decode round-trip for an instruction
static void test_roundtrip(moond_decoded_instr orig) {
    test_count++;
    
    // Encode
    moond_encode_result encoded = moond_encode_instr(&orig);
    
    if (encoded.status != 0) {
        printf("FAIL: Encode failed for %s (status=%u)\n", 
               moond_instr_mnemonic(orig.type), encoded.status);
        fail_count++;
        return;
    }
    
    // Decode
    moond_decoded_instr decoded = moond_decode_instr(encoded.word, orig.requires_extend);
    
    // Check round-trip
    bool matches = (decoded.type == orig.type &&
                   decoded.addr_mode == orig.addr_mode &&
                   decoded.address == orig.address &&
                   decoded.requires_extend == orig.requires_extend);
    
    // Check decoder status (should be 0 for exactly one match)
    if (decoded.status != 0) {
        printf("FAIL: Decoder status=%u (expected 0) for %s\n",
               decoded.status, moond_instr_mnemonic(orig.type));
        fail_count++;
        return;
    }
    
    if (matches) {
        pass_count++;
    } else {
        printf("FAIL: Round-trip mismatch for %s\n", moond_instr_mnemonic(orig.type));
        printf("  Original: type=%d mode=%d addr=0x%04x extend=%d\n",
               orig.type, orig.addr_mode, orig.address, orig.requires_extend);
        printf("  Decoded:  type=%d mode=%d addr=0x%04x extend=%d\n",
               decoded.type, decoded.addr_mode, decoded.address, decoded.requires_extend);
        fail_count++;
    }
}

int main(void) {
    printf("AGC Block-2 Encoder/Decoder Test\n");
    printf("=================================\n\n");
    
    // Test all instruction types with representative addresses
    
    // Basic K-type instructions (12-bit address)
    test_roundtrip((moond_decoded_instr){INSTR_TC, ADDR_K, 0x123, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_CA, ADDR_K, 0x456, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_CS, ADDR_K, 0x789, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_AD, ADDR_K, 0xABC, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_MSK, ADDR_K, 0xDEF, false, false, 0, 0xFF, 0});
    
    // Special addressless instructions
    test_roundtrip((moond_decoded_instr){INSTR_EXTEND, ADDR_NONE, 0, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_INHINT, ADDR_NONE, 0, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_RELINT, ADDR_NONE, 0, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_RESUME, ADDR_NONE, 0, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_GO, ADDR_NONE, 0, false, false, 0, 0xFF, 0});
    
    // 9-bit address instructions
    test_roundtrip((moond_decoded_instr){INSTR_DCA, ADDR_K, 0x123, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_DCS, ADDR_K, 0x1FF, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_NDX, ADDR_K, 0x0AB, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_MP, ADDR_K, 0x100, true, false, 0, 0xFF, 0});
    
    // Channel instructions (6-bit address)
    test_roundtrip((moond_decoded_instr){INSTR_READ, ADDR_H, 0x10, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_WRITE, ADDR_H, 0x20, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_RAND, ADDR_H, 0x30, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_ROR, ADDR_H, 0x3F, false, false, 0, 0xFF, 0});
    
    // Extracode channel instructions
    test_roundtrip((moond_decoded_instr){INSTR_WAND, ADDR_H, 0x15, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_WOR, ADDR_H, 0x25, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_RXOR, ADDR_H, 0x35, true, false, 0, 0xFF, 0});
    
    // Extracode E-type instructions (6-bit address)
    test_roundtrip((moond_decoded_instr){INSTR_CCS, ADDR_E, 0x10, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_DAS, ADDR_E, 0x20, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_LXCH, ADDR_E, 0x30, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_INCR, ADDR_E, 0x15, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_ADS, ADDR_E, 0x25, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_DXCH, ADDR_E, 0x35, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_TS, ADDR_E, 0x0A, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_XCH, ADDR_E, 0x1A, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_DV, ADDR_E, 0x2A, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_MSU, ADDR_E, 0x05, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_QXCH, ADDR_E, 0x15, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_AUG, ADDR_E, 0x25, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_DIM, ADDR_E, 0x35, true, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_SU, ADDR_E, 0x3F, true, false, 0, 0xFF, 0});
    
    // F-type branch instructions (6-bit address)
    test_roundtrip((moond_decoded_instr){INSTR_TCF, ADDR_F, 0x10, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_BZF, ADDR_F, 0x20, false, false, 0, 0xFF, 0});
    test_roundtrip((moond_decoded_instr){INSTR_BZMF, ADDR_F, 0x30, false, false, 0, 0xFF, 0});
    
    // NDX extracode variant
    test_roundtrip((moond_decoded_instr){INSTR_NDX, ADDR_E, 0x3F, true, false, 0, 0xFF, 0});
    
    printf("\n=================================\n");
    printf("Test Results:\n");
    printf("  Total:  %d\n", test_count);
    printf("  Passed: %d\n", pass_count);
    printf("  Failed: %d\n", fail_count);
    printf("=================================\n");
    
    return (fail_count == 0) ? 0 : 1;
}
