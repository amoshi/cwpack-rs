# Benchmark methodology — cwpack-rs vs original CWPack

## Workload (identical C and Rust)

Per timed run (`ITERATIONS=1_000_000`, default):

1. **Pack loop:** 1e6 × (`unsigned` in 0..65535, `str "bench"`, `nil`), reusing a 64KiB buffer (cursor reset each trio).
2. **Unpack loop:** pack one fixed 3-item message, then unpack it 1e6 times (3 `unpack_next` each).

Ops/run = `2 * 3 * ITERATIONS` = 6e6.

Sources:

- C: `bench/c_bench.c` linked with `../CWPack/src/cwpack.c` (`-O3`)
- Rust: `examples/rust_bench.rs` safe API (`cargo build --release`)

## Procedure

```bash
# from cwpack-rs root; CWPack checkout next to it (or CWPACK_SRC=...)
chmod +x bench/run.sh
./bench/run.sh
```

Defaults: `WARMUP=2`, `RUNS=20`. Override with env vars.

Script writes `bench/results.json` with:

- **p50_ms / p99_ms / mean_ms** wall time (monotonic clock)
- **throughput_ops_per_s** = ops_per_run / mean_seconds
- **rss_kb** from `/usr/bin/time -l` (macOS) maximum RSS
- **startup_ms** p99 of in-process `init`/first nil (not process spawn)

## Honesty

- Same machine, consecutive C then Rust series.
- Regressions reported as-is.
- Source pin: `833fec93903f047ae5c47936f884ba27fc4c7a4c`

## JSON → MessagePack differential tests

Separate from the micro-benchmark above: prove **byte-identical** MessagePack output from original CWPack C and cwpack-rs on real(ish) JSON fixtures.

### Pipeline

1. `extra-tests/json_to_ops.py` walks JSON with **sorted object keys** and emits a deterministic op stream (`NIL`, `BOOL`, `U64`/`I64`, `F64BITS`, `STR`, `ARR`, `MAP`).
2. `extra-tests/ops_pack_c.c` packs ops via original `cwpack.c` (`-O3`).
3. `examples/ops_pack.rs` packs the same ops via the Rust safe API.
4. `cmp` the resulting `.mp` blobs.

Shared ops isolate the **codec** from JSON parser differences (one Python walk for both sides).

### Procedure

```bash
# from cwpack-rs root; needs ../CWPack (or CWPACK_SRC=...)
make json-diff
# or:
./extra-tests/run_json_diff.sh

# optional large fixtures (cities.json + US IP aggregates):
INCLUDE_LARGE=1 ./extra-tests/run_json_diff.sh
```

Artifacts land under `extra-tests/out/` (`*.ops`, `*.c.mp`, `*.rs.mp`).

### Fixtures

| File | Role |
|------|------|
| `extra-tests/fixtures/mixed_types.json` | nil/bool/int/float/str/unicode/empty/nested |
| `extra-tests/fixtures/nested_config.json` | deeper maps/arrays |
| `extra-tests/fixtures/events.json` | mixed records + null |
| `extra-tests/fixtures/cities_numeric_5k.json` | 5k cities, float lat/lng |
| `extra-tests/countries.json` | strict JSON country list |
| `extra-tests/cities.json` | large all-string geo (`INCLUDE_LARGE=1`) |
| `extra-tests/country-ip-blocks/…` | CIDR maps (`INCLUDE_LARGE=1`) |

### Number rules (ops emitter)

- JSON ints → `U64` / `I64`
- Finite integer-valued floats in i64/u64 range → int ops
- Other floats → `F64BITS` (native IEEE754 bit pattern) for exact C/Rust match

Pass criterion: **all compared fixtures byte-identical** (script exits 0). Details: `extra-tests/README.md`.

## Library API

How to call the codec from Rust or C: [`docs/API.md`](../docs/API.md) and the crate [`README.md`](../README.md) (“API — how to use”).
