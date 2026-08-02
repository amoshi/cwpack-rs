/* Practical demo: cw_pack_insert sticky-error bug corrupts a real MessagePack
 * object so a consumer unpacks the WRONG field value (not a clean failure).
 *
 * Story (delayed error-check, as CWPack README encourages):
 *   Build map { "status": true, "payload": <ext> } in compatibility mode.
 *   EXT is illegal → sticky ILLEGAL_CALL after the "payload" key is written.
 *   App still calls cw_pack_insert() to splice a fallback fragment.
 *   Stock C appends raw bytes anyway. A best-effort sender ships start..current.
 *   Receiver unpacks: status=true, payload=66  ('B' of "BUG!" as fixint) —
 *   NOT an error, NOT the intended payload.
 *
 * Exit 1 = bug reproduced (wrong decoded payload).
 * Exit 2 = setup failed.
 * Exit 0 = upstream fixed insert (no corrupt append).
 */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "cwpack.h"

static int fail_setup(const char *msg) {
    fprintf(stderr, "setup failed: %s\n", msg);
    return 2;
}

static void dump_hex(const uint8_t *p, size_t n) {
    size_t i;
    printf("  wire (%zu bytes):", n);
    for (i = 0; i < n; i++)
        printf(" %02x", p[i]);
    printf("\n");
}

int main(void) {
    uint8_t buf[64];
    uint8_t ext_payload[1] = {0x99};
    cw_pack_context pc;
    cw_unpack_context uc;
    size_t n;

    /* Quick proof: compatibility forbids EXT (so the failure below is real). */
    memset(buf, 0xAA, sizeof buf);
    cw_pack_context_init(&pc, buf, sizeof buf, 0);
    cw_pack_set_compatibility(&pc, true);
    cw_pack_ext(&pc, 1, ext_payload, 1);
    if (pc.return_code != CWP_RC_ILLEGAL_CALL || (pc.current - pc.start) != 0)
        return fail_setup("compat mode must reject EXT with ILLEGAL_CALL");
    printf("C OK: compatibility forbids EXT (sticky ILLEGAL_CALL)\n\n");

    /* --- Build a realistic object with delayed error checking --- */
    memset(buf, 0, sizeof buf);
    cw_pack_context_init(&pc, buf, sizeof buf, 0);
    cw_pack_set_compatibility(&pc, true);

    printf("App encodes map { status: true, payload: <ext> } (compat ON)\n");
    cw_pack_map_size(&pc, 2);
    cw_pack_str(&pc, "status", 6);
    cw_pack_boolean(&pc, true);
    cw_pack_str(&pc, "payload", 7);
    /* Value for payload: EXT — illegal in compat → sticky error, no write. */
    cw_pack_ext(&pc, 1, ext_payload, 1);
    printf("  after failed ext: return_code=%d packed=%ld\n",
           pc.return_code, (long)(pc.current - pc.start));
    if (pc.return_code != CWP_RC_ILLEGAL_CALL)
        return fail_setup("expected sticky ILLEGAL_CALL after ext");

    /*
     * Fallback: splice pre-encoded bytes with insert (common for nested blobs).
     * README says this must be a no-op after error. Stock C still writes.
     */
    printf("  fallback: cw_pack_insert(\"BUG!\") while rc is still sticky error\n");
    cw_pack_insert(&pc, "BUG!", 4);
    n = (size_t)(pc.current - pc.start);
    printf("  after insert: return_code=%d packed=%zu\n", pc.return_code, n);
    dump_hex(buf, n);

    if (n < 4 || memcmp(buf + n - 4, "BUG!", 4) != 0) {
        printf("C OK?: insert did not append (upstream fixed?)\n");
        return 0;
    }

    /*
     * Best-effort / buggy caller: ship bytes even though return_code != OK
     * (also what you get if a wrapper returns len without checking rc).
     * Receiver unpacks the MessagePack object.
     */
    printf("\nSender ships start..current despite return_code=%d\n", pc.return_code);
    printf("Receiver unpacks:\n");

    cw_unpack_context_init(&uc, buf, (unsigned long)n, 0);

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_MAP || uc.item.as.map.size != 2)
        return fail_setup("unpack map header");

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_STR)
        return fail_setup("unpack key status");
    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_BOOLEAN || !uc.item.as.boolean)
        return fail_setup("unpack status=true");
    printf("  status  = true\n");

    cw_unpack_next(&uc);
    if (uc.return_code || uc.item.type != CWP_ITEM_STR)
        return fail_setup("unpack key payload");
    cw_unpack_next(&uc);
    /* 'B' (0x42) is a positive fixint → silently becomes integer 66 */
    if (uc.return_code == 0 && uc.item.type == CWP_ITEM_POSITIVE_INTEGER) {
        printf("  payload = %llu   ← WRONG (expected EXT/bin or hard failure)\n",
               (unsigned long long)uc.item.as.u64);
        printf("  ('B' of ASCII \"BUG!\" was decoded as MessagePack fixint 66)\n");
        printf("\nC BUG CONFIRMED: user-visible data corruption via sticky insert\n");
        return 1;
    }

    printf("unexpected unpack of payload: type=%d rc=%d\n",
           (int)uc.item.type, uc.return_code);
    return 2;
}
