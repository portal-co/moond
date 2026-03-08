#pragma once

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// AGC Block-2 Disassembler
// Converts 15-bit machine words back to assembly text.
//
// Output format per line:
//   MNEMONIC            — for no-address instructions
//   MNEMONIC 0oADDR     — for addressed instructions

typedef struct {
    char* text;         // heap-allocated null-terminated string
    const char* error;  // NULL on success
} moond_disasm_line;

// Disassemble a single word.
// extend_state tracks whether the previous word was EXTEND.
// Updates *extend_state after each call.
moond_disasm_line moond_disasm_word(uint16_t word, bool* extend_state);

// Free resources in a moond_disasm_line.
void moond_disasm_line_free(moond_disasm_line* l);

// Disassemble an array of words.
// Returns a heap-allocated null-terminated array of strings.
// Caller must free each string and the array with moond_disasm_free.
char** moond_disasm_words(const uint16_t* words, size_t count);

// Free the array returned by moond_disasm_words.
void moond_disasm_free(char** lines, size_t count);
