# Verification & benchmark methodology — cwpack-rs

How to prove and measure the port under the current design:

- **Native safe Rust only** (`#![forbid(unsafe_code)]`) — no C FFI / no linking of `cwpack_module_test.c` into Rust.
- **C-shaped API** (`CwPackContext` + `cw_pack_*` / `cw_unpack_*`) implemented in Rust.
- **Original CWPack C** is an **oracle** (sibling checkout), used for byte-diff, cross-language roundtrip, latent-bug demos, and latency compare — never as a dependency of the Rust crate.

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

# 3) Cross-language file roundtrip (pack ↔ unpack across languages)
make cross-roundtrip
# Rust → .mp → C unpack+check; C → .mp → Rust unpack+check; bytes identical

# 4) Latent C bug (Bug Catcher): sticky insert → wrong decoded field
make sticky-insert
# C: receiver sees payload=66; Rust: honest unpack error (no fake int)

# 5) Performance: C oracle vs Rust (p50/p99/RSS/throughput)
make bench
# writes bench/results.json

# 6) Differential self-fuzz (Rust pack↔unpack roundtrip)
make fuzz
# or: CWPACK_FUZZ_SECS=60 cargo run --release --example fuzz_harness | tee fuzz/log.txt
```

| Step | Command | What it proves | Needs `../CWPack`? |
|------|---------|----------------|--------------------|
| Smoke | `cargo test --release` | `cw_pack_*` API works | no |
| JSON diff | `make json-diff` | C and Rust emit **same `.mp` bytes** | **yes** |
| Cross file | `make cross-roundtrip` | Rust↔C pack/unpack via files | **yes** |
| Sticky insert | `make sticky-insert` | C: `payload=66` corruption; Rust: no fake field | **yes** |
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

## 3. Cross-language file roundtrip

**Purpose:** prove that MessagePack written by one language is correctly **unpacked and field-checked** by the other — not only that packers emit identical bytes (that is §2), but that each side’s unpack path understands the other’s file.

Complements `json-diff`: here the artifact is a real `.mp` file on disk, and verification walks types/values with `cw_unpack_next` (C oracle / Rust API).

### Canonical object (same layout both writers)

```
map(4):
  "compact" -> true
  "schema"  -> 0
  "name"    -> "demo"
  "vals"    -> array(3): -32, 255, nil
```

Key order is fixed so both packers produce the same encoding.

### Pipeline

1. **Rust pack → file** — `examples/cross_write.rs` (`cw_pack_*`) → `extra-tests/out/cross_from_rust.mp`
2. **C unpack + check** — `extra-tests/cross_read.c` + `CWPack/src/cwpack.c` reads the file, asserts each field, expects `CWP_RC_END_OF_INPUT`
3. **C pack → file** — `extra-tests/cross_write.c` → `extra-tests/out/cross_from_c.mp`
4. **Rust unpack + check** — `examples/cross_read.rs` mirrors the same field checks
5. **`cmp`** — both `.mp` files must be byte-identical

### Run

```bash
make cross-roundtrip
# equivalent:
./extra-tests/run_cross_roundtrip.sh
```

Needs `../CWPack` (or `CWPACK_SRC`). Artifacts under `extra-tests/out/`.

**Pass:** script exits 0; stderr shows `c verified OK` / `rust verified OK` and identical byte sizes.

---

## 4. Sticky `cw_pack_insert` bug (C broken / Rust fixed)

**Purpose:** document a **latent bug in upstream CWPack** found while porting (Port Mortem “Bug Catcher”), with a **user-visible wrong field** — not only “bytes were written”, but a receiver that unpacks a plausible incorrect value.

### Upstream contract (CWPack README)

1. **Backward compatibility:** with compatibility mode on, packing **EXT** (and TIMESTAMP) is illegal.
2. **Error handling:** once a context has an error, further calls are immediate no-ops (sticky `return_code`) so callers may batch packs and check once at the end.

Almost every `cw_pack_*` in `cwpack.c` starts with `if (pack_context->return_code) return;`.  
**`cw_pack_insert` does not** — after a sticky error it can still `memcpy` and advance `current`.

### Practical story (what the user sees)

App encodes `{ "status": true, "payload": <ext> }` with compatibility ON:

1. Pack map + `status`/`true` + key `payload` (17 bytes so far).
2. `cw_pack_ext` → sticky **`ILLEGAL_CALL` (-7)**; value not written.
3. Fallback `cw_pack_insert("BUG!")` while sticky error is set:
   - **Stock C:** appends `42 55 47 21` → **21 bytes** total.
   - **cwpack-rs:** no-op → stays **17 bytes** (truncated after the key).
4. Best-effort sender ships `start..current` despite `rc != OK`.
5. Receiver unpacks:
   - **C:** `status=true`, **`payload=66`** (`'B'` as MessagePack fixint) — silent wrong typed data.
   - **Rust:** `status=true`, then **unpack error** on `payload` (incomplete map) — no fake integer.

Sample C wire after the bug:

```text
82 a6 73 74 61 74 75 73 c3 a7 70 61 79 6c 6f 61 64 42 55 47 21
                                 ^-- key "payload" --^  B  U  G  !
```

### Run

```bash
make sticky-insert
# equivalent:
./extra-tests/run_sticky_insert.sh
```

| Side | Source | Expected |
|------|--------|----------|
| C | `extra-tests/sticky_insert_bug.c` + `CWPack/src/cwpack.c` | `payload = 66` + `C BUG CONFIRMED`, exit **1** |
| Rust | `examples/sticky_insert_ok.rs` | `payload = <error …>` + `Rust OK`, exit **0** |

Harness passes only if C still exhibits the corruption **and** Rust does not invent a fake field.  
If upstream adds the missing `return_code` check to `cw_pack_insert`, update this harness (C would exit 0).

Also recorded in [`DECISIONS.md`](../DECISIONS.md) §9.

---

## 5. Micro-benchmark (C oracle vs Rust)

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

## 6. Fuzz (Rust self-differential)

**Purpose:** pack→unpack roundtrip stress; optional Port Mortem “60s+” log.

```bash
make fuzz
# short local check:
CWPACK_FUZZ_SECS=3 cargo run --release --example fuzz_harness | tee fuzz/log.txt
```

Expect `divergences=0`. Log: `fuzz/log.txt`.

This does **not** call C; for C-vs-Rust use `make json-diff` / `make cross-roundtrip` / `make sticky-insert`.

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
