// Test program for AGC Block-2 instruction encoder/decoder
// Tests encode-decode round-trip and collision detection

#include "decode.h"
#include "encode.h"
#include <cstdint>
#include <stdbool.h>
#include <stdio.h>

static int test_count = 0;
static int pass_count = 0;
static int fail_count = 0;

// Test encode-decode round-trip for an instruction
static void test_roundtrip(uint16_t orig, bool requires_extend) {
  test_count++;

  // Decode
  moond_decoded_instr decoded = moond_decode_instr(orig, requires_extend);

  // Check decoder status (should be 0 for exactly one match)
  if (decoded.status != 0) {
    if (decoded.status + 1 == 0) {
      printf("WARN: invalid decode=%u", decoded.status);
      return;
    } else {
      printf("FAIL: Decoder status=%u (expected 0)\n", decoded.status);
      fail_count++;
      return;
    }
  }

  // Encode
  moond_encode_result encoded = moond_encode_instr(&decoded);

  if (encoded.status != 0) {
    printf("FAIL: Encode failed for %s (status=%u)\n",
           moond_instr_mnemonic(orig.type), encoded.status);
    fail_count++;
    return;
  }

  // Check round-trip
  bool matches =
      (decoded.type == encoded.type && decoded.addr_mode == encoded.addr_mode &&
       decoded.address == encoded.address &&
       decoded.requires_extend == encoded.requires_extend);

  if (matches) {
    pass_count++;
  } else {
    printf("FAIL: Round-trip mismatch for %s\n",
           moond_instr_mnemonic(orig.type));
    printf("  Original: type=%d mode=%d addr=0x%04x extend=%d\n", orig.type,
           orig.addr_mode, orig.address, orig.requires_extend);
    printf("  Decoded:  type=%d mode=%d addr=0x%04x extend=%d\n", decoded.type,
           decoded.addr_mode, decoded.address, decoded.requires_extend);
    fail_count++;
  }
}

int main(void) {
  printf("AGC Block-2 Encoder/Decoder Test\n");
  printf("=================================\n\n");
  for (uint16_t i = 0; i <= 0xffff; i++) {
    // Test all possible 16-bit instruction words
    test_roundtrip(i, false);
    test_roundtrip(i, true);
  }

  printf("\n=================================\n");
  printf("Test Results:\n");
  printf("  Total:  %d\n", test_count);
  printf("  Passed: %d\n", pass_count);
  printf("  Failed: %d\n", fail_count);
  printf("=================================\n");

  return (fail_count == 0) ? 0 : 1;
}
