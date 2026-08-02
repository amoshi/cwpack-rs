/* Read MessagePack file and verify canonical object (C reader for Rust writer).
 * Usage: cross_read_c <in.mp>
 */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "cwpack.h"

static int fail(const char *msg) {
    fprintf(stderr, "c verify FAIL: %s\n", msg);
    return 1;
}

static int expect_str(cw_unpack_context *uc, const char *want) {
    cw_unpack_next(uc);
    if (uc->return_code)
        return fail("unpack str");
    if (uc->item.type != CWP_ITEM_STR)
        return fail("type != str");
    if (uc->item.as.str.length != strlen(want))
        return fail("str length");
    if (memcmp(uc->item.as.str.start, want, uc->item.as.str.length) != 0)
        return fail("str value");
    return 0;
}

static int verify(const uint8_t *buf, size_t len) {
    cw_unpack_context uc;
    if (cw_unpack_context_init(&uc, buf, (unsigned long)len, 0))
        return fail("init");

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_MAP || uc.item.as.map.size != 4)
        return fail("map header");

    if (expect_str(&uc, "compact"))
        return 1;
    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_BOOLEAN || !uc.item.as.boolean)
        return fail("compact != true");

    if (expect_str(&uc, "schema"))
        return 1;
    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_POSITIVE_INTEGER || uc.item.as.u64 != 0)
        return fail("schema != 0");

    if (expect_str(&uc, "name"))
        return 1;
    if (expect_str(&uc, "demo"))
        return 1;

    if (expect_str(&uc, "vals"))
        return 1;
    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_ARRAY || uc.item.as.array.size != 3)
        return fail("vals array");

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_NEGATIVE_INTEGER || uc.item.as.i64 != -32)
        return fail("vals[0]");

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_POSITIVE_INTEGER || uc.item.as.u64 != 255)
        return fail("vals[1]");

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_NIL)
        return fail("vals[2]");

    cw_unpack_next(&uc);
    if (uc.return_code != CWP_RC_END_OF_INPUT)
        return fail("expected END_OF_INPUT");

    fprintf(stderr, "c verified OK (%zu bytes)\n", len);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <in.mp>\n", argv[0]);
        return 2;
    }
    FILE *f = fopen(argv[1], "rb");
    if (!f) {
        perror("fopen");
        return 3;
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        perror("fseek");
        fclose(f);
        return 4;
    }
    long sz = ftell(f);
    if (sz < 0) {
        perror("ftell");
        fclose(f);
        return 4;
    }
    rewind(f);
    uint8_t *buf = malloc((size_t)sz);
    if (!buf) {
        fclose(f);
        return 5;
    }
    if (fread(buf, 1, (size_t)sz, f) != (size_t)sz) {
        perror("fread");
        free(buf);
        fclose(f);
        return 6;
    }
    fclose(f);
    int rc = verify(buf, (size_t)sz);
    free(buf);
    return rc;
}
