/* Identical workload for C CWPack vs cwpack-rs.
 * Ops per run: ITERATIONS * (pack unsigned + pack str + pack nil + unpack×3)
 */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "cwpack.h"

#ifndef ITERATIONS
#define ITERATIONS 1000000
#endif

static double now_ms(void) {
#if defined(__APPLE__)
    return (double)clock_gettime_nsec_np(CLOCK_MONOTONIC_RAW) / 1e6;
#else
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1e3 + (double)ts.tv_nsec / 1e6;
#endif
}

static void workload(uint8_t *buf, size_t buflen) {
    cw_pack_context pc;
    cw_unpack_context uc;
    const char *s = "bench";
    uint32_t slen = 5;

    cw_pack_context_init(&pc, buf, (unsigned long)buflen, 0);
    for (int i = 0; i < ITERATIONS; i++) {
        cw_pack_unsigned(&pc, (uint64_t)(i & 0xffff));
        cw_pack_str(&pc, s, slen);
        cw_pack_nil(&pc);
        if (pc.return_code) {
            fprintf(stderr, "pack rc=%d at i=%d\n", pc.return_code, i);
            exit(2);
        }
        /* reset buffer cursor each item trio to keep buffer small */
        pc.current = pc.start;
    }

    /* one packed message for unpack loop */
    cw_pack_context_init(&pc, buf, (unsigned long)buflen, 0);
    cw_pack_unsigned(&pc, 42);
    cw_pack_str(&pc, s, slen);
    cw_pack_nil(&pc);
    unsigned long packed = (unsigned long)(pc.current - pc.start);

    for (int i = 0; i < ITERATIONS; i++) {
        cw_unpack_context_init(&uc, buf, packed, 0);
        cw_unpack_next(&uc);
        cw_unpack_next(&uc);
        cw_unpack_next(&uc);
        if (uc.return_code) {
            fprintf(stderr, "unpack rc=%d at i=%d\n", uc.return_code, i);
            exit(3);
        }
    }
}

/* Mode:
 *   timed  — print elapsed_ms for one workload (stdout: one float)
 *   startup — init + one nil pack, print elapsed_ms
 */
int main(int argc, char **argv) {
    const char *mode = argc > 1 ? argv[1] : "timed";
    uint8_t buf[65536];

    if (strcmp(mode, "startup") == 0) {
        double t0 = now_ms();
        cw_pack_context pc;
        cw_pack_context_init(&pc, buf, sizeof buf, 0);
        cw_pack_nil(&pc);
        double t1 = now_ms();
        printf("%.6f\n", t1 - t0);
        return pc.return_code ? 1 : 0;
    }

    double t0 = now_ms();
    workload(buf, sizeof buf);
    double t1 = now_ms();
    printf("%.6f\n", t1 - t0);
    return 0;
}
