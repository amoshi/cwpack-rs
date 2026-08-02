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

Preferred surface matches CWPack C names so examples almost copy-paste.

| Surface | What | When |
|---------|------|------|
| **C-like Rust** | `CwPackContext` + `cw_pack_*` / `cw_unpack_*` | Application code |
| **Low-level** | `cwpack::pack` / `unpack` (`encode_*`) | Custom cursors |
| **C ABI** | `ffi` / `utils` + `include/*.h` | Link original C tests |

### Cargo

```toml
[dependencies]
cwpack = { path = "." }
```

### Pack (same names as C)

C:

```c
cw_pack_context pc;
char buffer[32];
cw_pack_context_init(&pc, buffer, 32, 0);
cw_pack_map_size(&pc, 2);
cw_pack_str(&pc, "compact", 7);
cw_pack_boolean(&pc, true);
cw_pack_str(&pc, "schema", 6);
cw_pack_unsigned(&pc, 0);
```

Rust (free functions — closest paste):

```rust
use cwpack::{
    cw_pack_boolean, cw_pack_map_size, cw_pack_str, cw_pack_unsigned, CwPackContext,
};

let mut buffer = [0u8; 32];
let mut pc = CwPackContext::new(&mut buffer);

cw_pack_map_size(&mut pc, 2);
cw_pack_str(&mut pc, b"compact", 7);
cw_pack_boolean(&mut pc, true);
cw_pack_str(&mut pc, b"schema", 6);
cw_pack_unsigned(&mut pc, 0);

assert_eq!(pc.return_code, 0);
assert_eq!(pc.len_packed(), 18);
```

Same names as **methods** on the context:

```rust
pc.cw_pack_map_size(2);
pc.cw_pack_str(b"compact", 7);
pc.cw_pack_boolean(true);
```

| C | Rust |
|---|------|
| `cw_pack_nil/true/false/boolean` | same |
| `cw_pack_signed/unsigned` | same (`i64` / `u64`) |
| `cw_pack_float/double` | same |
| `cw_pack_array_size/map_size` | same |
| `cw_pack_str/bin(ctx, ptr, len)` | `cw_pack_str(&mut pc, bytes, len)` |
| `cw_pack_ext/time/insert` | same |
| `cw_pack_set_compatibility` | same |
| sticky `pc.return_code` | `pc.return_code` (`0` = ok) |

### Unpack

```rust
use cwpack::{cw_look_ahead, cw_unpack_next, CwUnpackContext, ItemType};

let mut uc = CwUnpackContext::new(packed_bytes);
let t = cw_look_ahead(&mut uc);
cw_unpack_next(&mut uc);
if uc.return_code == 0 && uc.item.type_code == ItemType::Str as i32 {
    let s = uc.item_blob();
}
// or: uc.cw_unpack_next();
```

Also: `cw_skip_items(&mut uc, count)`.

### Errors

Sticky `return_code` like C. Numeric codes match `CWP_RC_*` (`cwpack::Error`). Low-level `encode_*` APIs return `Result` instead.

### C ABI (link from C)

```bash
cargo build --release
clang -O2 -I include your.c target/release/libcwpack.a \
  -framework Security -framework CoreFoundation   # macOS
```

See `run-module-test.sh`.

### Examples in-tree

| Example | Purpose |
|---------|---------|
| `examples/rust_bench.rs` | micro-bench |
| `examples/ops_pack.rs` | JSON differential ops → msgpack |
| `examples/fuzz_harness.rs` | self fuzz |

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
