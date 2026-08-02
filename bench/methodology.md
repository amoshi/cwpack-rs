# Verification & benchmark methodology — cwpack-rs

How to prove and measure the port under the current design:

- **Native safe Rust only** (`#![forbid(unsafe_code)]`) — no C FFI / no linking of `cwpack_module_test.c` into Rust.
- **C-shaped API** (`CwPackContext` + `cw_pack_*` / `cw_unpack_*`) implemented in Rust.
- **Original CWPack C** is an **oracle** (sibling checkout), used for byte-diff and latency compare — never as a dependency of the Rust crate.

Source pin: `833fec93903f047ae5c47936f884ba27fc4c7a4c`  
Layout: `cwpack-rs/` next to `CWPack/` (or set `CWPACK_SRC`).

---

## Quick checklist (recommended order)

From `cwpack-rs` root:

```bash
# 0) prerequisites
# - Rust toolchain (rustc/cargo)
# - clang (for C oracle benches / ops_pack_c)
# - python3
# - ../CWPack at pinned SHA (or CWPACK_SRC=/path/to/CWPack)

cargo build --release

# 1) Unit / API smoke (native Rust, C-like names)
cargo test --release
# or: make test

# 2) Behavioral equivalence: identical MessagePack bytes vs C oracle
make json-diff
# optional large fixtures:
INCLUDE_LARGE=1 ./extra-tests/run_json_diff.sh

# 3) Performance: C oracle vs Rust (p50/p99/RSS/throughput)
make bench
# writes bench/results.json

# 4) Differential self-fuzz (Rust pack↔unpack roundtrip)
make fuzz
# or: CWPACK_FUZZ_SECS=60 cargo run --release --example fuzz_harness | tee fuzz/log.txt
```

| Step | Command | What it proves | Needs `../CWPack`? |
|------|---------|----------------|--------------------|
| Smoke | `cargo test --release` | `cw_pack_*` API works | no |
| JSON diff | `make json-diff` | C and Rust emit **same `.mp` bytes** | **yes** |
| Bench | `make bench` | latency / RSS / throughput vs C | **yes** |
| Fuzz | `make fuzz` | self roundtrip, 60s, 0 divergences | no |

There is **no** `cargo build --features c-abi` and **no** linking of upstream `cwpack_module_test.c` against Rust. Upstream test file may remain under `tests/original/` as a hashed reference only.

---

## 1. Smoke tests (`cargo test`)

**Purpose:** exercise the public C-like Rust API.

```bash
cargo test --release
```

Covers homepage-style pack (`cw_pack_map_size` / `cw_pack_str` / …), method form (`pc.cw_pack_*`), and a nil pack→unpack path. See `tests/smoke.rs`.

API docs: [`docs/API.md`](../docs/API.md), [`README.md`](../README.md).

---

## 2. JSON → MessagePack differential (primary equivalence test)

**Purpose:** prove **byte-identical** MessagePack from original CWPack C and cwpack-rs on shared fixtures. This replaces “link C module test to Rust cdylib”.

### Pipeline

1. `extra-tests/json_to_ops.py` — parse JSON, **sort object keys**, emit op stream  
   (`NIL`, `BOOL`, `U64`/`I64`, `F64BITS`, `STR`, `ARR`, `MAP`).
2. `extra-tests/ops_pack_c.c` + `../CWPack/src/cwpack.c` (`-O3`) → `.c.mp`
3. `examples/ops_pack.rs` (Rust `pack::encode_*`) → `.rs.mp`
4. `cmp` — must match.

One Python walk → both codecs; JSON parser differences do not affect the compare.

### Run

```bash
make json-diff
# equivalent:
./extra-tests/run_json_diff.sh

INCLUDE_LARGE=1 ./extra-tests/run_json_diff.sh   # cities.json + US IP blocks
```

Artifacts: `extra-tests/out/` (`*.ops`, `*.c.mp`, `*.rs.mp`).

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
- Other floats → `F64BITS` (native IEEE754 bits) for exact C/Rust match

**Pass:** script exits 0, every fixture prints `OK`. Details: `extra-tests/README.md`.

---

## 3. Micro-benchmark (C oracle vs Rust)

**Purpose:** honest p50/p99/mean, RSS, throughput, startup — same workload both sides.

### Workload (identical C and Rust)

Per timed run (`ITERATIONS=1_000_000`, default):

1. **Pack loop:** 1e6 × (`unsigned` in 0..65535, `str "bench"`, `nil`), 64KiB buffer, cursor reset each trio.
2. **Unpack loop:** one fixed 3-item message, unpack 1e6 times (3 nexts each).

Ops/run = `2 * 3 * ITERATIONS` = 6e6.

| Side | Source |
|------|--------|
| C | `bench/c_bench.c` + `CWPACK_SRC/src/cwpack.c` (`-O3`) |
| Rust | `examples/rust_bench.rs` (`cargo build --release --example rust_bench`) |

### Run

```bash
make bench
# or:
./bench/run.sh
```

Env overrides: `RUNS` (default 20), `WARMUP` (default 2), `ITERATIONS`, `CWPACK_SRC`.

Writes **`bench/results.json`**:

- `p50_ms` / `p99_ms` / `mean_ms`
- `throughput_ops_per_s` = ops_per_run / mean_seconds
- `rss_kb` via `/usr/bin/time -l` (macOS) max RSS
- `startup_ms` p99 of in-process init+nil (not process spawn)
- machine / clang / rustc metadata

### Honesty

- Same machine; C series then Rust series back-to-back.
- Report regressions; do not hide slower results.
- This is **not** the upstream MPack/CMP comparative harness.

---

## 4. Fuzz (Rust self-differential)

**Purpose:** pack→unpack roundtrip stress; optional Port Mortem “60s+” log.

```bash
make fuzz
# short local check:
CWPACK_FUZZ_SECS=3 cargo run --release --example fuzz_harness | tee fuzz/log.txt
```

Expect `divergences=0`. Log: `fuzz/log.txt`.

This does **not** call C; for C-vs-Rust use `make json-diff`.

---

## What we deliberately do not run

| Approach | Status |
|----------|--------|
| Link `tests/original/cwpack_module_test.c` against Rust `.a`/`.dylib` | **Removed** (would need `unsafe` C ABI) |
| Rust calling into original `libcwpack` as implementation | **Forbidden** (contest Rule §05) |
| Editing hashed originals under `tests/original/` | **Avoid** — keep as reference only |

---

## Library API (for writing more tests)

- Public: `CwPackContext` / `CwUnpackContext` + `cw_pack_*` / `cw_unpack_*` (sticky `return_code`).
- Low-level: `pack::encode_*` / `unpack::*` (`Result`).
- Docs: [`docs/API.md`](../docs/API.md), [`README.md`](../README.md).
