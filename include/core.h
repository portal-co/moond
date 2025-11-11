#pragma once

#include <stdint.h>
#define OCTAL(a) a
typedef enum moond_bl2_reg {
  A = OCTAL(00),
  L = OCTAL(01),
  Q = OCTAL(02),
  EB = OCTAL(03),
  FB = OCTAL(04),
  Z = OCTAL(05),
  BB = OCTAL(06),
  $ZERO = OCTAL(07),
} moond_bl2_reg;

#define moond_bl2_mem_start OCTAL(010)
#define moond_bl2_mem_pin_start OCTAL(04000)

typedef uint16_t moomd_bl2_word_t;

typedef struct moond_bl2_bamk {
  uint8_t erasable_bank_reg;
  uint8_t fixed_bank_reg;
  uint8_t fixed_extension_bits;
  moomd_bl2_word_t sreg_start;
  uint8_t sreg_len_div_0o400;
  uint8_t sreg_tile_div_0o400;
} moond_bl2_bamk;

#define moond_bl2_no_bank 0xff

extern moond_bl2_bamk moond_bl2_bank_kind(moomd_bl2_word_t addr);
extern moomd_bl2_word_t moond_bl2_get_bb_from_eb_and_fb(moomd_bl2_word_t eb,
                                                        moomd_bl2_word_t fb);
extern void moond_bl2_get_eb_and_fb_from_bb(moomd_bl2_word_t bb,
                                            moomd_bl2_word_t *eb,
                                            moomd_bl2_word_t *fb);

static inline moomd_bl2_word_t moond_bl2_bound(moomd_bl2_word_t x) {
  return (x >> 1) << 1;
}