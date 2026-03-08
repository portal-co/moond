#include "disasm.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    bool extend_state = false;
    char line[128];

    while (fgets(line, sizeof(line), stdin)) {
        // Strip newline
        size_t n = strlen(line);
        while (n > 0 && (line[n-1] == '\n' || line[n-1] == '\r')) {
            line[--n] = '\0';
        }

        // Skip blank lines
        if (n == 0) continue;

        // Parse hex word: accept 0xNNNN or plain NNNN
        char* end = NULL;
        unsigned long val = strtoul(line, &end, 16);
        if (end == line || (*end != '\0' && *end != '\n')) {
            fprintf(stderr, "agc_disasm: invalid hex word: %s\n", line);
            return 1;
        }

        if (val > 0x7FFF) {
            fprintf(stderr, "agc_disasm: word out of 15-bit range: 0x%lx\n", val);
            return 1;
        }

        moond_disasm_line dl = moond_disasm_word((uint16_t)val, &extend_state);
        if (dl.error) {
            fprintf(stderr, "agc_disasm: %s\n", dl.error);
            moond_disasm_line_free(&dl);
            return 1;
        }

        printf("%s\n", dl.text ? dl.text : "");
        moond_disasm_line_free(&dl);
    }

    return 0;
}
