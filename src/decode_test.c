// Test program for AGC Block-2 instruction encoder/decoder
// Tests encode-decode round-trip and collision detection

#include "decode.h"
#include "encode.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

static int test_count = 0;
static int pass_count = 0;
static int fail_count = 0;

// Test encode-decode round-trip for an instruction
static void test_roundtrip(uint16_t orig_word, bool requires_extend) {
  test_count++;

  // Decode
  moond_instr decoded = moond_decode_instr(orig_word, requires_extend);

  // Check decoder status (should be 0 for exactly one match)
  if (decoded.status != 0) {
    if (decoded.status + 1 == 0) {
      // No match, skip silently
      return;
    } else {
      printf("FAIL: Decoder status=%d (expected 0) for word=0x%04x/0o%06o "
             "(reversed "
             "0o%06o) extend=%d\n",
             decoded.status, orig_word, orig_word,
             extract_agc_bits_reversed(orig_word, 1, 15), requires_extend);
      fail_count++;
      return;
    }
  }

  // Encode
  moond_encode_result encoded = moond_encode_instr(&decoded);

  if (!encoded.success) {
    printf("FAIL: Encode failed for %s: %s\n",
           moond_instr_mnemonic(decoded.type),
           encoded.error ? encoded.error : "unknown error");
    fail_count++;
    return;
  }

  // Check round-trip
  if (encoded.word == orig_word) {
    pass_count++;
  } else {
    printf("FAIL: Round-trip mismatch for %s\n",
           moond_instr_mnemonic(decoded.type));
    printf("  Original: word=0x%04x/0o%06o (reversed 0o%06o) extend=%d\n",
           orig_word, orig_word, extract_agc_bits_reversed(orig_word, 1, 15),
           requires_extend);
    printf("  Decoded:  type=%d mode=%d addr=0x%04x extend=%d\n", decoded.type,
           decoded.addr_mode, decoded.address, decoded.requires_extend);
    printf("  Encoded:  word=0x%04x/0o%06o (reversed 0o%06o)\n", encoded.word,
           encoded.word, extract_agc_bits_reversed(encoded.word, 1, 15));
    fail_count++;
  }
}

int main(void) {
  printf("AGC Block-2 Encoder/Decoder Test\n");
  printf("=================================\n\n");
  for (uint32_t i = 0; i <= 0xffff; i++) {
    printf("INFO: trying %04x\n", (uint16_t)i);
    // Test all possible 16-bit instruction words
    test_roundtrip((uint16_t)i, false);
    test_roundtrip((uint16_t)i, true);
  }

  printf("\n=================================\n");
  printf("Test Results:\n");
  printf("  Total:  %d\n", test_count);
  printf("  Passed: %d\n", pass_count);
  printf("  Failed: %d\n", fail_count);
  printf("=================================\n");

  return (fail_count == 0) ? 0 : 1;
}
