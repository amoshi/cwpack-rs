/* Write canonical MessagePack object to a file (C → Rust reader).
 * Usage: cross_write_c <out.mp>
 *
 * map(4):
 *   "compact" -> true
 *   "schema"  -> 0
 *   "name"    -> "demo"
 *   "vals"    -> array(3): -32, 255, nil
 */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "cwpack.h"

static int pack_object(uint8_t *buf, size_t buflen, size_t *out_len) {
    cw_pack_context pc;
    if (cw_pack_context_init(&pc, buf, (unsigned long)buflen, 0))
        return pc.return_code;

    cw_pack_map_size(&pc, 4);

    cw_pack_str(&pc, "compact", 7);
    cw_pack_boolean(&pc, true);

    cw_pack_str(&pc, "schema", 6);
    cw_pack_unsigned(&pc, 0);

    cw_pack_str(&pc, "name", 4);
    cw_pack_str(&pc, "demo", 4);

    cw_pack_str(&pc, "vals", 4);
    cw_pack_array_size(&pc, 3);
    cw_pack_signed(&pc, -32);
    cw_pack_unsigned(&pc, 255);
    cw_pack_nil(&pc);

    if (pc.return_code)
        return pc.return_code;
    *out_len = (size_t)(pc.current - pc.start);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <out.mp>\n", argv[0]);
        return 2;
    }
    uint8_t buf[256];
    size_t n = 0;
    int rc = pack_object(buf, sizeof buf, &n);
    if (rc) {
        fprintf(stderr, "pack failed rc=%d\n", rc);
        return 3;
    }
    FILE *f = fopen(argv[1], "wb");
    if (!f) {
        perror("fopen");
        return 4;
    }
    if (fwrite(buf, 1, n, f) != n) {
        perror("fwrite");
        fclose(f);
        return 5;
    }
    fclose(f);
    fprintf(stderr, "c wrote %zu bytes -> %s\n", n, argv[1]);
    return 0;
}
