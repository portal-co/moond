#pragma once

#include <stdint.h>
#include <stddef.h>

// AGC Block-2 Assembler
// Converts human-readable AGC assembly text to 15-bit machine words.
//
// Text format (one instruction per line):
//   # comment
//   EXTEND
//   CA  0o200       # load A from address 0200 octal
//   AD  0o300
//   CCS 0o50        # extracode — automatically prepend EXTEND word
//   TC  0o4500
//
// Rules:
// - Lines starting with '#' are comments (skip)
// - Blank lines are skipped
// - Mnemonic is the first token (case-insensitive)
// - Optional address is second token: decimal, 0oNNN octal, or 0NNN C octal
// - Mnemonics that requires_extend automatically emit an EXTEND word first
// - Fixed-address instructions ignore any user-provided address

typedef struct {
    uint16_t* words;    // heap-allocated array of words (caller frees)
    size_t count;       // number of words
    const char* error;  // NULL on success, error string on failure
} moond_asm_result;

// Assemble text to words. Returns moond_asm_result.
// On success, result.error is NULL and result.words/count are valid.
// Caller must free result.words.
moond_asm_result moond_assemble(const char* text);

// Free resources in a moond_asm_result.
void moond_asm_result_free(moond_asm_result* r);
