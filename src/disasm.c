#include "disasm.h"
#include "instr.h"
#include "decode.h"

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ---------------------------------------------------------------------------
// moond_disasm_word
// ---------------------------------------------------------------------------
moond_disasm_line moond_disasm_word(uint16_t word, bool* extend_state) {
    moond_disasm_line line = {.text = NULL, .error = NULL};

    if (!extend_state) {
        line.error = "NULL extend_state pointer";
        return line;
    }

    bool was_extend = *extend_state;
    moond_decoded_instr instr = moond_decode_instr(word, was_extend);

    // Update extend state: set if this word is EXTEND, clear otherwise
    *extend_state = (instr.type == INSTR_EXTEND);

    if (instr.type == INSTR_UNKNOWN) {
        // Emit a raw hex comment for unknown words
        char buf[32];
        snprintf(buf, sizeof(buf), "# UNKNOWN 0x%04x", word);
        line.text = strdup(buf);
        if (!line.text) line.error = "Out of memory";
        return line;
    }

    const char* mnemonic = moond_instr_mnemonic(instr.type);
    char buf[64];

    if (instr.addr_mode == ADDR_NONE) {
        // No address field
        snprintf(buf, sizeof(buf), "%s", mnemonic);
    } else {
        // Print address as octal with 0o prefix, zero-padded to 4 digits
        snprintf(buf, sizeof(buf), "%s 0o%04o", mnemonic, (unsigned)instr.address);
    }

    line.text = strdup(buf);
    if (!line.text) line.error = "Out of memory";
    return line;
}

// ---------------------------------------------------------------------------
// moond_disasm_line_free
// ---------------------------------------------------------------------------
void moond_disasm_line_free(moond_disasm_line* l) {
    if (!l) return;
    free(l->text);
    l->text = NULL;
    l->error = NULL;
}

// ---------------------------------------------------------------------------
// moond_disasm_words
// ---------------------------------------------------------------------------
char** moond_disasm_words(const uint16_t* words, size_t count) {
    if (!words && count > 0) return NULL;

    // Allocate array of (count+1) pointers, last is NULL sentinel
    char** lines = calloc(count + 1, sizeof(char*));
    if (!lines) return NULL;

    bool extend_state = false;
    for (size_t i = 0; i < count; i++) {
        moond_disasm_line dl = moond_disasm_word(words[i], &extend_state);
        if (dl.error || !dl.text) {
            // On error, fill remaining with NULL and return partial result
            lines[i] = NULL;
            return lines;
        }
        lines[i] = dl.text; // transfer ownership
    }
    lines[count] = NULL;
    return lines;
}

// ---------------------------------------------------------------------------
// moond_disasm_free
// ---------------------------------------------------------------------------
void moond_disasm_free(char** lines, size_t count) {
    if (!lines) return;
    for (size_t i = 0; i < count; i++) {
        free(lines[i]);
    }
    free(lines);
}
