# Extra JSON → MessagePack differential tests

Pipeline (isolates the codec from JSON parser differences):

1. `json_to_ops.py` walks JSON with **sorted object keys** and emits an op stream.
2. `ops_pack_c` packs ops with original CWPack C.
3. `examples/ops_pack` packs the same ops with cwpack-rs.
4. `cmp` the MessagePack bytes.

```bash
chmod +x extra-tests/run_json_diff.sh
./extra-tests/run_json_diff.sh

# also cities.json + US IP aggregates (slower):
INCLUDE_LARGE=1 ./extra-tests/run_json_diff.sh
```

## Fixtures

| File | Role |
|------|------|
| `fixtures/mixed_types.json` | nil/bool/int/float/str/unicode/empty/nested |
| `fixtures/nested_config.json` | deeper maps/arrays |
| `fixtures/events.json` | mixed records + null |
| `fixtures/cities_numeric_5k.json` | 5k cities with float lat/lng |
| `countries.json` | strict JSON country list (converted from JS-object notation) |
| `cities.json` | large all-string geo (optional via `INCLUDE_LARGE`) |
| `country-ip-blocks/` | CIDR maps (optional; nested clone) |

## Number rules

- JSON ints → `U64` / `I64`
- Finite floats that are integer-valued in i64/u64 range → int ops
- Other floats → `F64BITS` (IEEE754 bit pattern) for exact C/Rust match
