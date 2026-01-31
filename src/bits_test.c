#include "bits.h"
#include "core.h"
#include <stdio.h>
#include <stdbool.h>

static bool test_bit_conversion(void) {
    printf("Testing bit number conversion...\n");
    
    // AGC bit 1 = MSB = C bit 14
    if (agc_to_c_bit(1) != 14) {
        printf("  FAIL: AGC bit 1 -> C bit %d (expected 14)\n", agc_to_c_bit(1));
        return false;
    }
    
    // AGC bit 15 = LSB = C bit 0
    if (agc_to_c_bit(15) != 0) {
        printf("  FAIL: AGC bit 15 -> C bit %d (expected 0)\n", agc_to_c_bit(15));
        return false;
    }
    
    // AGC bit 7 = C bit 8
    if (agc_to_c_bit(7) != 8) {
        printf("  FAIL: AGC bit 7 -> C bit %d (expected 8)\n", agc_to_c_bit(7));
        return false;
    }
    
    printf("  ✓ Bit conversion works\n");
    return true;
}

static bool test_extract_bits(void) {
    printf("\nTesting bit extraction...\n");
    
    // Test word: 030050 octal = CA K 00050
    // Bits 1-3 (opcode) should be 03 octal
    // Bits 4-15 (address) should be 00050 octal
    uint16_t word = 030050;
    
    uint16_t opcode = extract_agc_bits(word, 1, 3);
    if (opcode != 03) {
        printf("  FAIL: Extract bits 1-3 from %05o: got %02o (expected 03)\n", word, opcode);
        return false;
    }
    
    uint16_t addr = extract_agc_bits(word, 4, 15);
    if (addr != 00050) {
        printf("  FAIL: Extract bits 4-15 from %05o: got %05o (expected 00050)\n", word, addr);
        return false;
    }
    
    // Test word: 017234 octal = MP K 0234
    // Bits 1-6 (opcode) should be 17 octal
    // Bits 7-15 (address) should be 0234 octal
    word = 017234;
    
    uint16_t opcode6 = extract_agc_bits(word, 1, 6);
    if (opcode6 != 017) {
        printf("  FAIL: Extract bits 1-6 from %05o: got %02o (expected 17)\n", word, opcode6);
        return false;
    }
    
    uint16_t addr9 = extract_agc_bits(word, 7, 15);
    if (addr9 != 0234) {
        printf("  FAIL: Extract bits 7-15 from %05o: got %04o (expected 0234)\n", word, addr9);
        return false;
    }
    
    // Test word: 010030 octal = READ H 030
    // Bits 1-6 (opcode) should be 10 octal
    // Bits 7-9 (quarter) should be 0 octal
    // Bits 10-15 (address) should be 030 octal
    word = 010030;
    
    uint16_t opcode_read = extract_agc_bits(word, 1, 6);
    if (opcode_read != 010) {
        printf("  FAIL: Extract bits 1-6 from %05o: got %02o (expected 10)\n", word, opcode_read);
        return false;
    }
    
    uint16_t quarter = extract_agc_bits(word, 7, 9);
    if (quarter != 0) {
        printf("  FAIL: Extract bits 7-9 from %05o: got %o (expected 0)\n", word, quarter);
        return false;
    }
    
    uint16_t addr6 = extract_agc_bits(word, 10, 15);
    if (addr6 != 030) {
        printf("  FAIL: Extract bits 10-15 from %05o: got %03o (expected 030)\n", word, addr6);
        return false;
    }
    
    printf("  ✓ Bit extraction works\n");
    return true;
}

static bool test_insert_bits(void) {
    printf("\nTesting bit insertion...\n");
    
    // Build CA K 00050 = 030050 octal
    // Insert 03 into bits 1-3, 00050 into bits 4-15
    uint16_t word = 0;
    word = insert_agc_bits(word, 03, 1, 3);
    word = insert_agc_bits(word, 00050, 4, 15);
    
    if (word != 030050) {
        printf("  FAIL: Built word %05o (expected 030050)\n", word);
        return false;
    }
    
    // Build READ H 030 = 010030 octal
    // Insert 10 into bits 1-6, 0 into bits 7-9, 030 into bits 10-15
    word = 0;
    word = insert_agc_bits(word, 010, 1, 6);
    word = insert_agc_bits(word, 0, 7, 9);
    word = insert_agc_bits(word, 030, 10, 15);
    
    if (word != 010030) {
        printf("  FAIL: Built word %05o (expected 010030)\n", word);
        return false;
    }
    
    printf("  ✓ Bit insertion works\n");
    return true;
}

static bool test_make_bits(void) {
    printf("\nTesting make_bits...\n");
    
    // Make CA opcode in bits 1-3
    uint16_t word = make_agc_bits(03, 1, 3);
    if (word != 030000) {
        printf("  FAIL: make_agc_bits(03, 1, 3) = %05o (expected 030000)\n", word);
        return false;
    }
    
    // Make READ opcode in bits 1-6
    word = make_agc_bits(010, 1, 6);
    if (word != 010000) {
        printf("  FAIL: make_agc_bits(010, 1, 6) = %05o (expected 010000)\n", word);
        return false;
    }
    
    printf("  ✓ make_bits works\n");
    return true;
}

static bool test_bit_reversal(void) {
    printf("\nTesting bit reversal...\n");
    
    // Reverse 0b000001 (6 bits) should give 0b100000 (32 decimal)
    uint16_t rev = reverse_bits(0b000001, 6);
    if (rev != 0b100000) {
        printf("  FAIL: reverse_bits(0b000001, 6) = 0b%06b (expected 0b100000)\n", rev);
        return false;
    }
    
    // Reverse 0b101 (3 bits) should give 0b101 (palindrome)
    rev = reverse_bits(0b101, 3);
    if (rev != 0b101) {
        printf("  FAIL: reverse_bits(0b101, 3) = 0b%03b (expected 0b101)\n", rev);
        return false;
    }
    
    // Reverse 0b001 (3 bits) should give 0b100
    rev = reverse_bits(0b001, 3);
    if (rev != 0b100) {
        printf("  FAIL: reverse_bits(0b001, 3) = 0b%03b (expected 0b100)\n", rev);
        return false;
    }
    
    // Reverse 01 octal = 1 decimal = 0b000001 (6 bits) -> 0b100000 = 040 octal = 32 decimal
    rev = reverse_bits(OCTAL(01), 6);
    if (rev != OCTAL(040)) {
        printf("  FAIL: reverse_bits(01 octal, 6) = %03o octal (expected 040 octal)\n", rev);
        return false;
    }
    
    printf("  ✓ Bit reversal works\n");
    return true;
}

static bool test_agc_semantic_operations(void) {
    printf("\nTesting AGC-semantic operations (with reversal)...\n");
    
    // Test AGC-semantic extraction:
    // If AGC documentation says order code "01" octal is in bits 1-6:
    // "01" octal = 1 decimal = 0b000001
    // In AGC bits, this means: bit 1 (MSB of field) = 0, bit 6 (LSB of field) = 1
    // So the bits must be stored REVERSED in the C representation
    // In C: bits 14-9, with bit 14 = 0 (AGC bit 1) and bit 9 = 1 (AGC bit 6)
    // That's 0b100000 in C bits 14-9
    uint16_t word = ((uint16_t)0b100000) << 9;  // Put reversed value in C bits 14-9
    uint16_t extracted = extract_agc_bits_reversed(word, 1, 6);
    if (extracted != OCTAL(01)) {
        printf("  FAIL: extract_agc_bits_reversed gave %02o (expected 01 octal)\n", extracted);
        printf("        word = %05o, C bits 14-9 = 0b%06b\n", word, (word >> 9) & 0x3F);
        return false;
    }
    
    // Test AGC-semantic insertion:
    // Insert opcode "01" octal into AGC bits 1-6
    // Should reverse it to 0b100000 and place in C bits 14-9
    word = insert_agc_bits_reversed(0, OCTAL(01), 1, 6);
    uint16_t c_bits = (word >> 9) & 0x3F;
    if (c_bits != 0b100000) {
        printf("  FAIL: insert_agc_bits_reversed gave C bits = 0b%06b (expected 0b100000)\n", c_bits);
        return false;
    }
    
    // Verify round-trip: extract what we inserted
    extracted = extract_agc_bits_reversed(word, 1, 6);
    if (extracted != OCTAL(01)) {
        printf("  FAIL: round-trip failed: inserted 01, extracted %02o\n", extracted);
        return false;
    }
    
    printf("  ✓ AGC-semantic operations work\n");
    return true;
}

int main(void) {
    printf("AGC Bit Manipulation Helper Test\n");
    printf("=================================\n\n");
    
    bool all_pass = true;
    all_pass &= test_bit_conversion();
    all_pass &= test_extract_bits();
    all_pass &= test_insert_bits();
    all_pass &= test_make_bits();
    all_pass &= test_bit_reversal();
    all_pass &= test_agc_semantic_operations();
    
    printf("\n");
    if (all_pass) {
        printf("✓ All tests passed!\n");
        return 0;
    } else {
        printf("✗ Some tests failed\n");
        return 1;
    }
}
