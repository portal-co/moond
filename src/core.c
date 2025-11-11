#include "core.h"

moond_bl2_bamk moond_bl2_bank_kind(moomd_bl2_word_t addr) {
  if (addr < OCTAL(01400))
    return (moond_bl2_bamk){.erasable_bank_reg = 0,
                            .fixed_bank_reg = moond_bl2_no_bank,
                            .fixed_extension_bits = moond_bl2_no_bank,
                            .sreg_start = OCTAL(00),
                            .sreg_len_div_0o400 = OCTAL(014),
                            .sreg_tile_div_0o400 = OCTAL(020)};
  if (addr < OCTAL(03777))
    return (moond_bl2_bamk){.erasable_bank_reg = (uint8_t)(addr >> 8),
                            .fixed_bank_reg = moond_bl2_no_bank,
                            .fixed_extension_bits = moond_bl2_no_bank,
                            .sreg_start = OCTAL(01400),
                            .sreg_len_div_0o400 = OCTAL(01),
                            .sreg_tile_div_0o400 = OCTAL(01)};
  if (addr < OCTAL(07777))
    return (moond_bl2_bamk){.erasable_bank_reg = moond_bl2_no_bank,
                            .fixed_bank_reg = moond_bl2_no_bank,
                            .fixed_extension_bits = moond_bl2_no_bank,
                            .sreg_start = OCTAL(04000),
                            .sreg_len_div_0o400 = OCTAL(010),
                            .sreg_tile_div_0o400 = OCTAL(010)};
  uint8_t bank = (uint8_t)((addr - OCTAL(010000)) >> 10);
  if (bank >= OCTAL(040)) {
    bank -= OCTAL(010);
    return (moond_bl2_bamk){.erasable_bank_reg = moond_bl2_no_bank,
                            .fixed_bank_reg = bank,
                            .fixed_extension_bits = 4,
                            .sreg_start = OCTAL(02000),
                            .sreg_len_div_0o400 = OCTAL(02),
                            .sreg_tile_div_0o400 = OCTAL(02)};
  }
  return (moond_bl2_bamk){.erasable_bank_reg = moond_bl2_no_bank,
                          .fixed_bank_reg = bank,
                          .fixed_extension_bits = 3,
                          .sreg_start = OCTAL(02000),
                          .sreg_len_div_0o400 = OCTAL(02),
                          .sreg_tile_div_0o400 = OCTAL(02)};
}
moomd_bl2_word_t moond_bl2_get_bb_from_eb_and_fb(moomd_bl2_word_t eb,
                                                 moomd_bl2_word_t fb) {
  return (fb & ((1 << 10) - 1)) | (eb >> 8 & 7);
}
void moond_bl2_get_eb_and_fb_from_bb(moomd_bl2_word_t bb, moomd_bl2_word_t *eb,
                                     moomd_bl2_word_t *fb) {
  *eb = (bb & 7) << 8;
  *fb = ((bb >> 10) & 31) << 10;
}