# Extra differential tests (C oracle vs cwpack-rs)

Needs sibling `../CWPack` (or `CWPACK_SRC`). Full write-up: [`bench/methodology.md`](../bench/methodology.md).

## JSON → MessagePack (`make json-diff`)

Pipeline (isolates the codec from JSON parser differences):

1. `json_to_ops.py` walks JSON with **sorted object keys** and emits an op stream.
2. `ops_pack_c` packs ops with original CWPack C.
3. `examples/ops_pack` packs the same ops with cwpack-rs.
4. `cmp` the MessagePack bytes.

```bash
./extra-tests/run_json_diff.sh
INCLUDE_LARGE=1 ./extra-tests/run_json_diff.sh   # cities + IP blocks
```

### Fixtures

| File | Role |
|------|------|
| `fixtures/mixed_types.json` | nil/bool/int/float/str/unicode/empty/nested |
| `fixtures/nested_config.json` | deeper maps/arrays |
| `fixtures/events.json` | mixed records + null |
| `fixtures/cities_numeric_5k.json` | 5k cities with float lat/lng |
| `countries.json` | strict JSON country list |
| `cities.json` | large all-string geo (`INCLUDE_LARGE`) |
| `country-ip-blocks/` | CIDR maps (`INCLUDE_LARGE`) |

### Number rules

- JSON ints → `U64` / `I64`
- Finite integer-valued floats in i64/u64 range → int ops
- Other floats → `F64BITS` (IEEE754 bits) for exact C/Rust match

## Cross-language file roundtrip (`make cross-roundtrip`)

Rust packs a canonical object to `.mp` → C unpacks/checks fields; then C packs → Rust unpacks/checks; `cmp` both files.

- `examples/cross_*.rs`, `extra-tests/cross_*.c`, `run_cross_roundtrip.sh`

## Sticky `cw_pack_insert` bug (`make sticky-insert`)

**User-visible corruption** in stock CWPack (Rust fixed):

1. Encode `{ "status": true, "payload": <ext> }` with compatibility ON → EXT → sticky `ILLEGAL_CALL`.
2. Fallback `cw_pack_insert("BUG!")`:
   - **C:** still appends (`… 42 55 47 21`, 21 bytes).
   - **Rust:** no-op (17 bytes, truncated after key `payload`).
3. Best-effort send of `start..current` → unpack:
   - **C:** `status=true`, **`payload=66`** (`'B'` as fixint) — wrong data.
   - **Rust:** decode **error** on `payload` — no fake integer.

| File | Role |
|------|------|
| `sticky_insert_bug.c` | C oracle; exit **1** = bug (`payload=66`) |
| `../examples/sticky_insert_ok.rs` | Rust; exit **0** = honest failure |
| `run_sticky_insert.sh` | harness |

See [`DECISIONS.md`](../DECISIONS.md) §9 · [`bench/methodology.md`](../bench/methodology.md) §4.
