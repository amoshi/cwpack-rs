/* Read ops stream from stdin/file, pack with CWPack C, write raw MessagePack to stdout. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#include "cwpack.h"

#define BUF_CAP (64 * 1024 * 1024)

static uint8_t *buf;
static size_t buf_cap;
static cw_pack_context pc;

static void die(const char *msg) {
    fprintf(stderr, "ops_pack_c: %s\n", msg);
    exit(1);
}

static int read_line(FILE *in, char *line, size_t n) {
    if (!fgets(line, (int)n, in))
        return 0;
    size_t L = strlen(line);
    if (L && line[L - 1] == '\n')
        line[L - 1] = 0;
    return 1;
}

static void ensure_init(void) {
    if (!buf) {
        buf_cap = BUF_CAP;
        buf = malloc(buf_cap);
        if (!buf)
            die("malloc");
        if (cw_pack_context_init(&pc, buf, (unsigned long)buf_cap, 0))
            die("pack init");
    }
}

int main(int argc, char **argv) {
    FILE *in = stdin;
    if (argc > 1) {
        in = fopen(argv[1], "rb");
        if (!in)
            die("open ops");
    }
    ensure_init();

    char line[4096];
    while (read_line(in, line, sizeof line)) {
        if (line[0] == 0)
            continue;
        if (strcmp(line, "NIL") == 0) {
            cw_pack_nil(&pc);
        } else if (strncmp(line, "BOOL ", 5) == 0) {
            cw_pack_boolean(&pc, atoi(line + 5) != 0);
        } else if (strncmp(line, "U64 ", 4) == 0) {
            cw_pack_unsigned(&pc, strtoull(line + 4, NULL, 10));
        } else if (strncmp(line, "I64 ", 4) == 0) {
            cw_pack_signed(&pc, strtoll(line + 4, NULL, 10));
        } else if (strncmp(line, "F64BITS ", 8) == 0) {
            uint64_t bits = strtoull(line + 8, NULL, 10);
            double d;
            memcpy(&d, &bits, 8);
            /* MessagePack wants big-endian bit pattern of IEEE754; cw_pack_double uses host bits */
            cw_pack_double(&pc, d);
        } else if (strncmp(line, "STR ", 4) == 0) {
            unsigned long len = strtoul(line + 4, NULL, 10);
            char *s = malloc(len + 1);
            if (!s)
                die("malloc str");
            if (fread(s, 1, len, in) != len)
                die("short str");
            int nl = fgetc(in);
            if (nl != '\n' && nl != EOF)
                die("str trailer");
            cw_pack_str(&pc, s, (uint32_t)len);
            free(s);
        } else if (strncmp(line, "ARR ", 4) == 0) {
            cw_pack_array_size(&pc, (uint32_t)strtoul(line + 4, NULL, 10));
        } else if (strncmp(line, "MAP ", 4) == 0) {
            cw_pack_map_size(&pc, (uint32_t)strtoul(line + 4, NULL, 10));
        } else {
            fprintf(stderr, "unknown op: %s\n", line);
            return 2;
        }
        if (pc.return_code) {
            fprintf(stderr, "pack rc=%d\n", pc.return_code);
            return 3;
        }
    }

    size_t n = (size_t)(pc.current - pc.start);
    if (fwrite(pc.start, 1, n, stdout) != n)
        die("write");
    if (in != stdin)
        fclose(in);
    return 0;
}
