# cwpack-rs

Safe Rust port of [clwi/CWPack](https://github.com/clwi/CWPack) (ANSI-C MessagePack encoder/decoder) for **Port Mortem 2026** — Track **A (C → Rust)**.

Source pin: `833fec93903f047ae5c47936f884ba27fc4c7a4c`

## Why this port

CWPack is a no-alloc, buffer-oriented MessagePack codec with sticky error codes and overflow/underflow handlers. The interesting C→Rust work is not “call c2rust”, but:

1. Prove behavioral equivalence against the **unmodified** C module test via a thin C ABI.
2. Keep the **core logic in safe Rust** (`Result`, `to_be_bytes`, no type-punning).
3. Confine `unsafe` to the FFI boundary only.

## Build (one command)

```bash
cargo build --release
```

Produces `target/release/libcwpack.{a,dylib,rlib}`.

Or via Docker:

```bash
docker build -t cwpack-rs .
```

## API — how to use

Two surfaces:

| Surface | Module | When |
|---------|--------|------|
| **Safe Rust** | `cwpack::pack` / `cwpack::unpack` | Application code (preferred) |
| **C ABI** | `cwpack::ffi` (+ `utils`) | Link original C tests / C callers |

No heap allocation in the core path: you own the buffer; the API advances a cursor (`pos`) up to `end`.

### Cargo

Path dependency (this repo):

```toml
[dependencies]
cwpack = { path = "." }
```

Or once published, `cwpack = "0.1"`.

### Pack (safe Rust)

```rust
use cwpack::pack;

fn pack_homepage_example() -> Result<Vec<u8>, cwpack::Error> {
    let mut buf = [0u8; 32];
    let end = buf.len();
    let mut pos = 0;

    pack::encode_map_size(&mut buf, &mut pos, end, 2)?;
    pack::encode_str(&mut buf, &mut pos, end, b"compact", false)?;
    pack::encode_bool(&mut buf, &mut pos, end, true)?;
    pack::encode_str(&mut buf, &mut pos, end, b"schema", false)?;
    pack::encode_unsigned(&mut buf, &mut pos, end, 0)?;

    Ok(buf[..pos].to_vec()) // 18 bytes
}
```

`be_compatible: bool` on `encode_str` / `encode_bin` / `encode_ext` / `encode_time` mirrors CWPack’s compatibility mode (no str8 / bin→str / no ext&time).

| Function | MessagePack |
|----------|-------------|
| `encode_nil` / `encode_bool` | nil, bool |
| `encode_unsigned` / `encode_signed` | ints |
| `encode_float` / `encode_double` | float32/64 |
| `encode_str` / `encode_bin` / `encode_ext` | str, bin, ext |
| `encode_array_size` / `encode_map_size` | containers (then pack elements) |
| `encode_time` | timestamp ext (−1) |
| `encode_insert` | raw bytes into stream |

Errors: `Error::BufferOverflow` if `pos + need > end`.

### Unpack (safe Rust)

```rust
use cwpack::item::ItemType;
use cwpack::unpack;

fn unpack_one(buf: &[u8]) -> Result<(), cwpack::Error> {
    let mut pos = 0;
    let end = buf.len();
    let d = unpack::unpack_next(buf, &mut pos, end)?;

    match d.type_code {
        t if t == ItemType::Nil as i32 => {}
        t if t == ItemType::Boolean as i32 => {
            let _ = d.boolean;
        }
        t if t == ItemType::PositiveInteger as i32 => {
            let _ = d.u64;
        }
        t if t == ItemType::Str as i32 => {
            let s = &buf[d.blob_off..d.blob_off + d.blob_len as usize];
            let _ = s;
        }
        t if t == ItemType::Array as i32 || t == ItemType::Map as i32 => {
            // `d.size` elements (map: 2*size following items)
            let _ = d.size;
        }
        _ => {}
    }
    Ok(())
}
```

Also:

- `unpack::skip_items(buf, &mut pos, end, count)` — skip `count` top-level items (containers expand like CWPack).
- `unpack::look_ahead(buf, pos, end) -> Result<i32>` — next type code **without** consuming (same codes as CWPack / `ItemType`).

`Decoded` fields: `type_code`, `boolean`, `u64`/`i64`, `real`/`long_real`, `size`, `blob_off`/`blob_len`, `time_sec`/`time_nsec`. Blob payloads are slices of the input buffer at `blob_off`.

### Errors

```rust
use cwpack::{Error, Result};
```

Same numeric codes as C `CWP_RC_*` (`Error::code()` / `Error::from_code`). Idiomatic Rust uses `Result<T>`; the C ABI keeps sticky `return_code` on the context.

### C ABI (for C callers / original tests)

Build `staticlib`/`cdylib`, include headers from `include/`, link `libcwpack.a`:

```bash
cargo build --release
clang -O2 -I include your.c target/release/libcwpack.a \
  -framework Security -framework CoreFoundation   # macOS
```

Symbols match CWPack (`cw_pack_*`, `cw_unpack_*`, plus utils like `cw_unpack_next_signed32`). See `include/cwpack.h` and `run-module-test.sh`.

`unsafe` lives only in `ffi` / `utils`; pack/unpack core is safe.

### Examples in-tree

| Example / tool | Purpose |
|----------------|---------|
| `examples/rust_bench.rs` | micro-bench workload |
| `examples/ops_pack.rs` | pack op-stream from `extra-tests/json_to_ops.py` |
| `examples/fuzz_harness.rs` | self differential fuzz |

```bash
cargo run --release --example rust_bench -- timed
```

## Run original module test against the port

```bash
./run-module-test.sh
```

Expected: `CWPack module test completed, no errors detected`

Original test file hashes: `tests/original/SHA256SUMS`.

## Bench & JSON differential

See [`bench/methodology.md`](bench/methodology.md) (latency/RSS micro-bench + JSON→ops→MessagePack C vs Rust byte compare).

```bash
make bench
make json-diff
```

## Layout

```
src/           safe pack/unpack + C ABI (ffi/utils)
include/       original C headers (for linking the C test)
tests/original/  unmodified cwpack_module_test.c
tests/smoke.rs   Rust-side smoke tests
extra-tests/   JSON fixtures + differential harness
fuzz/          differential harness
bench/         methodology + results
DECISIONS.md   architectural divergences
```

## Scope

**In:** core `cwpack.c` API + `goodies/utils` (required by module test).  
**Out (documented):** ObjC/Swift bindings, dump tool, basic-contexts, numeric-extensions, MPack/CMP perf comparison binary.

## License

MIT (same as CWPack).
