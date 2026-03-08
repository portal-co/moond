#include "asm.h"
#include "instr.h"
#include "encode.h"

#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <stdio.h>

// ---------------------------------------------------------------------------
// Mnemonic-to-type table
// ---------------------------------------------------------------------------

typedef struct {
    const char* name;
    moond_instr_type type;
} mnemonic_entry;

static const mnemonic_entry mnemonic_table[] = {
    {"TC",     INSTR_TC},
    {"TCF",    INSTR_TCF},
    {"CCS",    INSTR_CCS},
    {"BZF",    INSTR_BZF},
    {"BZMF",   INSTR_BZMF},
    {"CA",     INSTR_CA},
    {"CS",     INSTR_CS},
    {"DCA",    INSTR_DCA},
    {"DCS",    INSTR_DCS},
    {"TS",     INSTR_TS},
    {"XCH",    INSTR_XCH},
    {"LXCH",   INSTR_LXCH},
    {"QXCH",   INSTR_QXCH},
    {"DXCH",   INSTR_DXCH},
    {"NDX",    INSTR_NDX},
    {"AD",     INSTR_AD},
    {"SU",     INSTR_SU},
    {"MP",     INSTR_MP},
    {"DV",     INSTR_DV},
    {"ADS",    INSTR_ADS},
    {"DAS",    INSTR_DAS},
    {"INCR",   INSTR_INCR},
    {"AUG",    INSTR_AUG},
    {"DIM",    INSTR_DIM},
    {"MSU",    INSTR_MSU},
    {"MSK",    INSTR_MSK},
    {"READ",   INSTR_READ},
    {"WRITE",  INSTR_WRITE},
    {"RAND",   INSTR_RAND},
    {"WAND",   INSTR_WAND},
    {"ROR",    INSTR_ROR},
    {"WOR",    INSTR_WOR},
    {"RXOR",   INSTR_RXOR},
    {"EXTEND", INSTR_EXTEND},
    {"INHINT", INSTR_INHINT},
    {"RELINT", INSTR_RELINT},
    {"RESUME", INSTR_RESUME},
    {"GO",     INSTR_GO},
};

static const size_t mnemonic_table_size =
    sizeof(mnemonic_table) / sizeof(mnemonic_table[0]);

// ---------------------------------------------------------------------------
// Fixed-address instructions that ignore any user-provided address
// ---------------------------------------------------------------------------
static bool is_fixed_address(moond_instr_type type) {
    return type == INSTR_EXTEND ||
           type == INSTR_INHINT ||
           type == INSTR_RELINT ||
           type == INSTR_RESUME ||
           type == INSTR_GO;
}

// ---------------------------------------------------------------------------
// Uppercase a token in-place
// ---------------------------------------------------------------------------
static void str_toupper(char* s) {
    for (; *s; s++) *s = (char)toupper((unsigned char)*s);
}

// ---------------------------------------------------------------------------
// Parse address token. Supports:
//   decimal        e.g. 512
//   0oNNN          octal (preferred AGC notation)
//   0NNN           C octal
// Returns true on success, sets *out.
// ---------------------------------------------------------------------------
static bool parse_address(const char* tok, uint16_t* out) {
    if (!tok || tok[0] == '\0') return false;

    char* end = NULL;
    long val;

    if (tok[0] == '0' && (tok[1] == 'o' || tok[1] == 'O')) {
        // 0oNNN — octal
        val = strtol(tok + 2, &end, 8);
    } else if (tok[0] == '0' && tok[1] != '\0' && isdigit((unsigned char)tok[1])) {
        // 0NNN — C octal
        val = strtol(tok, &end, 8);
    } else {
        // decimal
        val = strtol(tok, &end, 10);
    }

    if (end == tok || *end != '\0') return false;
    if (val < 0 || val > 0x7FFF) return false;

    *out = (uint16_t)val;
    return true;
}

// ---------------------------------------------------------------------------
// Dynamic word buffer helpers
// ---------------------------------------------------------------------------
typedef struct {
    uint16_t* data;
    size_t len;
    size_t cap;
} word_buf;

static bool buf_push(word_buf* b, uint16_t w) {
    if (b->len == b->cap) {
        size_t new_cap = b->cap ? b->cap * 2 : 16;
        uint16_t* tmp = realloc(b->data, new_cap * sizeof(uint16_t));
        if (!tmp) return false;
        b->data = tmp;
        b->cap = new_cap;
    }
    b->data[b->len++] = w;
    return true;
}

// ---------------------------------------------------------------------------
// moond_assemble
// ---------------------------------------------------------------------------
moond_asm_result moond_assemble(const char* text) {
    moond_asm_result result = {.words = NULL, .count = 0, .error = NULL};

    if (!text) {
        result.error = "NULL input text";
        return result;
    }

    // Pre-compute the EXTEND word once
    moond_encode_result extend_enc = moond_encode_simple(INSTR_EXTEND, 0);
    if (!extend_enc.success) {
        result.error = "Failed to encode EXTEND word";
        return result;
    }
    uint16_t extend_word = extend_enc.word;

    word_buf buf = {.data = NULL, .len = 0, .cap = 0};

    // Iterate over lines
    const char* p = text;
    while (*p) {
        // Find end of line
        const char* line_start = p;
        while (*p && *p != '\n') p++;
        size_t line_len = (size_t)(p - line_start);
        if (*p == '\n') p++; // consume newline

        // Copy line for tokenisation (mutate safely)
        char line[512];
        if (line_len >= sizeof(line)) line_len = sizeof(line) - 1;
        memcpy(line, line_start, line_len);
        line[line_len] = '\0';

        // Strip inline comment
        char* hash = strchr(line, '#');
        if (hash) *hash = '\0';

        // Tokenise: mnemonic [address]
        char* tok = strtok(line, " \t\r");
        if (!tok) continue; // blank line

        // Uppercase the mnemonic
        str_toupper(tok);

        // Look up mnemonic
        moond_instr_type itype = INSTR_UNKNOWN;
        for (size_t i = 0; i < mnemonic_table_size; i++) {
            if (strcmp(tok, mnemonic_table[i].name) == 0) {
                itype = mnemonic_table[i].type;
                break;
            }
        }
        if (itype == INSTR_UNKNOWN) {
            result.error = "Unknown mnemonic";
            free(buf.data);
            buf.data = NULL;
            return result;
        }

        // Parse optional address token
        char* addr_tok = strtok(NULL, " \t\r#");
        uint16_t address = 0;
        bool have_addr = false;
        if (addr_tok) {
            // Strip any trailing comment character just in case
            char* h2 = strchr(addr_tok, '#');
            if (h2) *h2 = '\0';
            if (addr_tok[0] != '\0') {
                if (!parse_address(addr_tok, &address)) {
                    result.error = "Invalid address token";
                    free(buf.data);
                    buf.data = NULL;
                    return result;
                }
                have_addr = true;
            }
        }
        (void)have_addr; // fixed-address instructions ignore it

        // Auto-insert EXTEND if required (and this isn't an explicit EXTEND line)
        if (moond_instr_needs_extend(itype)) {
            if (!buf_push(&buf, extend_word)) {
                result.error = "Out of memory";
                free(buf.data);
                buf.data = NULL;
                return result;
            }
        }

        // Encode the instruction
        moond_encode_result enc;
        if (is_fixed_address(itype)) {
            enc = moond_encode_simple(itype, 0); // address is ignored by encoder
        } else {
            enc = moond_encode_simple(itype, address);
        }

        if (!enc.success) {
            result.error = enc.error ? enc.error : "Encoding failed";
            free(buf.data);
            buf.data = NULL;
            return result;
        }

        if (!buf_push(&buf, enc.word)) {
            result.error = "Out of memory";
            free(buf.data);
            buf.data = NULL;
            return result;
        }
    }

    result.words = buf.data;
    result.count = buf.len;
    return result;
}

// ---------------------------------------------------------------------------
// moond_asm_result_free
// ---------------------------------------------------------------------------
void moond_asm_result_free(moond_asm_result* r) {
    if (!r) return;
    free(r->words);
    r->words = NULL;
    r->count = 0;
    r->error = NULL;
}
