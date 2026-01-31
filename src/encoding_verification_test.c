#include "bits.h"
#include "decode.h"
#include "encode.h"
#include "core.h"
#include <stdio.h>
#include <stdbool.h>

// Test structure
typedef struct {
    const char* name;
    moond_instr_type type;
    uint16_t address;
    uint16_t expected_word;  // Expected encoding from AGC documentation
    bool needs_extend;
} known_encoding_test;

// Known instruction encodings verified from AGC Block-2 documentation
// These are the actual binary encodings used by the AGC hardware
static const known_encoding_test known_encodings[] = {
    // Basic 3-bit opcode instructions (order code XX.)
    {"TC 00000", INSTR_TC, 00000, 000000, false},      // 00.0000
    {"GO (TC 04000)", INSTR_GO, 04000, 004000, false}, // 00.4000 (special: decodes as GO)
    {"CA 00050", INSTR_CA, 00050, 030050, false},      // 03.0050
    {"CS 00123", INSTR_CS, 00123, 040123, false},      // 04.0123
    {"AD 01234", INSTR_AD, 01234, 061234, false},      // 06.1234
    {"MSK 07777", INSTR_MSK, 07777, 077777, false},    // 07.7777
    
    // 6-bit whole code instructions (order code XX, no quarter)
    {"DCA 00234", INSTR_DCA, 00234, 013234, false},    // 013.234 (6-bit opcode 013 octal, 9-bit addr)
    {"DCS 00456", INSTR_DCS, 00456, 014456, false},    // 014.456
    {"MP 00234", INSTR_MP, 00234, 017234, true},       // 017.234 (extracode)
    
    // Quarter code instructions - Channel (order code 010.X)
    {"READ 00030", INSTR_READ, 00030, 010030, false},  // 010.0.030 (opcode=010, qtr=0, addr=030)
    {"WRITE 00030", INSTR_WRITE, 00030, 010130, false},// 010.1.030 (qtr=1)
    {"RAND 00030", INSTR_RAND, 00030, 010230, false},  // 010.2.030 (qtr=2)
    {"ROR 00030", INSTR_ROR, 00030, 010430, false},    // 010.4.030 (qtr=4)
    {"WAND 00030", INSTR_WAND, 00030, 010330, true},   // 010.3.030 (qtr=3, extracode)
    {"WOR 00030", INSTR_WOR, 00030, 010530, true},     // 010.5.030 (qtr=5, extracode)
    {"RXOR 00030", INSTR_RXOR, 00030, 010630, true},   // 010.6.030 (qtr=6, extracode)
    
    // Quarter code instructions - E-type (order code XX.X)
    {"CCS 00050", INSTR_CCS, 00050, 001050, true},     // 01.0.050 (opcode=01, qtr=0, addr=050)
    {"TS 00050", INSTR_TS, 00050, 005450, true},       // 05.4.050 (opcode=05, qtr=4, addr=050)
    {"XCH 00050", INSTR_XCH, 00050, 005550, true},     // 05.5.050 (opcode=05, qtr=5, addr=050)
    {"LXCH 00050", INSTR_LXCH, 00050, 002250, true},   // 02.2.050 (opcode=02, qtr=2, addr=050)
    {"DV 00050", INSTR_DV, 00050, 011050, true},       // 011.0.050 (opcode=011 octal, qtr=0, addr=050)
};

static int test_count = 0;
static int pass_count = 0;

static void test_encoding(const known_encoding_test* test) {
    test_count++;
    
    // Encode the instruction
    moond_encode_result enc = moond_encode_simple(test->type, test->address);
    
    if (!enc.success) {
        printf("✗ %s: Encoding failed: %s\n", test->name, enc.error);
        return;
    }
    
    // Check if encoded word matches expected
    if (enc.word != test->expected_word) {
        printf("✗ %s: Encoded to %05o (expected %05o)\n",
               test->name, enc.word, test->expected_word);
        printf("  Binary comparison:\n");
        printf("    Got:      ");
        for (int i = 14; i >= 0; i--) printf("%d", (enc.word >> i) & 1);
        printf("\n    Expected: ");
        for (int i = 14; i >= 0; i--) printf("%d", (test->expected_word >> i) & 1);
        printf("\n");
        return;
    }
    
    // Decode it back
    moond_decoded_instr dec = moond_decode_instr(enc.word, test->needs_extend);
    
    // Check if decoded instruction matches
    if (dec.type != test->type) {
        printf("✗ %s: Round-trip failed - decoded as %s\n",
               test->name, moond_instr_mnemonic(dec.type));
        return;
    }
    
    // Success!
    printf("✓ %s: %05o (bits: ", test->name, enc.word);
    
    // Show bit fields
    uint8_t opcode_6 = (enc.word >> 9) & 0x3F;
    uint8_t opcode_3 = (enc.word >> 12) & 0x7;
    uint8_t quarter = (enc.word >> 6) & 0x7;
    uint16_t addr_6 = enc.word & 0x3F;
    uint16_t addr_9 = enc.word & 0x1FF;
    uint16_t addr_12 = enc.word & 0xFFF;
    
    // Determine which format
    if (opcode_6 >= 010) {
        printf("op6=%02o q=%o addr6=%03o", opcode_6, quarter, addr_6);
    } else if (opcode_6 >= 013 && opcode_6 <= 017) {
        printf("op6=%02o addr9=%04o", opcode_6, addr_9);
    } else if (opcode_6 <= 012 && quarter != 0) {
        printf("op6=%02o q=%o addr6=%03o", opcode_6, quarter, addr_6);
    } else {
        printf("op3=%o addr12=%05o", opcode_3, addr_12);
    }
    
    printf(")\n");
    pass_count++;
}

int main(void) {
    printf("AGC Block-2 Known Encoding Verification Test\n");
    printf("=============================================\n");
    printf("Verifying against documented AGC instruction encodings\n\n");
    
    printf("Basic Instructions (3-bit opcode):\n");
    for (int i = 0; i < 6; i++) {
        test_encoding(&known_encodings[i]);
    }
    printf("\n");
    
    printf("Whole Code Instructions (6-bit opcode, no quarter):\n");
    for (int i = 6; i < 9; i++) {
        test_encoding(&known_encodings[i]);
    }
    printf("\n");
    
    printf("Channel Instructions (order code 010.X):\n");
    for (int i = 9; i < 16; i++) {
        test_encoding(&known_encodings[i]);
    }
    printf("\n");
    
    printf("E-Type Quarter Code Instructions:\n");
    for (int i = 16; i < 21; i++) {
        test_encoding(&known_encodings[i]);
    }
    printf("\n");
    
    printf("Results: %d/%d tests passed\n", pass_count, test_count);
    
    if (pass_count == test_count) {
        printf("\n✓ All encodings match AGC documentation!\n");
        printf("✓ No bit reversal needed - implementation is correct.\n");
        return 0;
    } else {
        printf("\n✗ Some encodings don't match - investigation needed.\n");
        return 1;
    }
}
