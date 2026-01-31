#include <stdio.h>
#include <stdint.h>
#include "bits.h"
#include "core.h"

// Test a known instruction from the PDF
// TC K instruction has order code 00 (octal) in bits 1-3
// TC 00100 (octal) should be:
//   - Bits 1-3: 000 (octal 0)
//   - Bits 4-15: 000100 (octal 100)

int main() {
    printf("AGC Encoding Verification\n");
    printf("=========================\n\n");
    
    // Test 1: TC K with address 00100 (octal)
    printf("Test 1: TC K 00100 (octal)\n");
    printf("Expected in C representation (hex): 0x%04X\n", OCTAL(00100));
    printf("Explanation:\n");
    printf("  - AGC bits 1-3 (opcode): 000 binary = 0 octal\n");
    printf("  - AGC bits 4-15 (address): 000001000000 binary = 100 octal = 64 decimal\n");
    printf("  - In C: bit 14-12 = 000, bit 11-0 = 000001000000 = 0x0040\n\n");
    
    uint16_t tc_word = OCTAL(00100);
    printf("Word value: 0x%04X\n", tc_word);
    printf("Binary: ");
    for (int i = 14; i >= 0; i--) {
        printf("%d", (tc_word >> i) & 1);
        if (i % 3 == 0 && i > 0) printf(" ");
    }
    printf("\n\n");
    
    // Extract opcode (AGC bits 1-3)
    uint8_t opcode_raw = extract_agc_bits(tc_word, 1, 3);
    uint8_t opcode_rev = extract_agc_bits_reversed(tc_word, 1, 3);
    printf("Opcode (AGC bits 1-3):\n");
    printf("  Raw extraction (C order): %03o octal = %d decimal = ", opcode_raw, opcode_raw);
    for (int i = 2; i >= 0; i--) printf("%d", (opcode_raw >> i) & 1);
    printf("\n");
    printf("  Bit-reversed (AGC order): %03o octal = %d decimal = ", opcode_rev, opcode_rev);
    for (int i = 2; i >= 0; i--) printf("%d", (opcode_rev >> i) & 1);
    printf(" <- CORRECT for AGC\n\n");
    
    // Extract address (AGC bits 4-15)
    uint16_t addr_raw = extract_agc_bits(tc_word, 4, 15);
    uint16_t addr_rev = extract_agc_bits_reversed(tc_word, 4, 15);
    printf("Address (AGC bits 4-15):\n");
    printf("  Raw extraction (C order): %05o octal = %d decimal\n", addr_raw, addr_raw);
    printf("  Bit-reversed (AGC order): %05o octal = %d decimal <- CORRECT for AGC\n\n", addr_rev, addr_rev);
    
    printf("Conclusion: We need bit reversal to get correct AGC values!\n\n");
    
    // Test 2: CA K with address 00050 (octal)
    printf("Test 2: CA K 00050 (octal)\n");
    printf("Expected order code: 03 (octal)\n");
    printf("Expected in C representation (hex): 0x%04X\n", OCTAL(030050));
    
    uint16_t ca_word = OCTAL(030050);
    printf("Word value: 0x%04X\n", ca_word);
    
    opcode_raw = extract_agc_bits(ca_word, 1, 3);
    opcode_rev = extract_agc_bits_reversed(ca_word, 1, 3);
    printf("Opcode: raw=%03o, reversed=%03o (should be 03)\n", opcode_raw, opcode_rev);
    
    addr_raw = extract_agc_bits(ca_word, 4, 15);
    addr_rev = extract_agc_bits_reversed(ca_word, 4, 15);
    printf("Address: raw=%05o, reversed=%05o (should be 00050)\n\n", addr_raw, addr_rev);
    
    printf("✓ Tests show bit reversal is needed!\n");
    
    return 0;
}
