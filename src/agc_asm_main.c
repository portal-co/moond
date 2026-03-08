#include "asm.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define READ_CHUNK 4096

int main(void) {
    // Read all of stdin into a heap buffer
    char* buf = NULL;
    size_t len = 0;
    size_t cap = 0;

    while (1) {
        if (len + READ_CHUNK + 1 > cap) {
            size_t new_cap = cap ? cap * 2 : READ_CHUNK * 2;
            char* tmp = realloc(buf, new_cap);
            if (!tmp) {
                fprintf(stderr, "agc_asm: out of memory\n");
                free(buf);
                return 1;
            }
            buf = tmp;
            cap = new_cap;
        }
        size_t n = fread(buf + len, 1, READ_CHUNK, stdin);
        len += n;
        if (n < READ_CHUNK) break;
    }
    if (buf) buf[len] = '\0';
    else buf = strdup("");

    moond_asm_result result = moond_assemble(buf);
    free(buf);

    if (result.error) {
        fprintf(stderr, "agc_asm: %s\n", result.error);
        return 1;
    }

    for (size_t i = 0; i < result.count; i++) {
        printf("0x%04x\n", (unsigned)result.words[i]);
    }

    moond_asm_result_free(&result);
    return 0;
}
