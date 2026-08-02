# cwpack-rs

Safe Rust port of [clwi/CWPack](https://github.com/clwi/CWPack) (ANSI-C MessagePack encoder/decoder) for **Port Mortem 2026** — Track **A (C → Rust)**.

Source pin: `833fec93903f047ae5c47936f884ba27fc4c7a4c`

## Why this port

CWPack is a no-alloc, buffer-oriented MessagePack codec. This port:

1. Keeps **C-shaped names** (`cw_pack_*` / `cw_unpack_*`) on native Rust contexts for easy migration.
2. Implements the codec **entirely in safe Rust** (`#![forbid(unsafe_code)]`) — no C FFI.
3. Proves equivalence via **differential MessagePack bytes** (ops harness + benches) against original CWPack C as an oracle.

## Build (one command)

```bash
cargo build --release
```

Produces `target/release/libcwpack.rlib`.

## API — how to use

Native Rust only. Names and arity match CWPack C so examples almost copy-paste.

| Surface | What |
|---------|------|
| **Public API** | `CwPackContext` / `CwUnpackContext` + `cw_pack_*` / `cw_unpack_*` |
| **Low-level** | `cwpack::pack` / `unpack` (`encode_*`) |

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

Rust:

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

Or methods: `pc.cw_pack_map_size(2)`.

| C | Rust |
|---|------|
| `cw_pack_nil/true/false/boolean` | same |
| `cw_pack_signed/unsigned` | same (`i64` / `u64`) |
| `cw_pack_float/double` | same |
| `cw_pack_array_size/map_size` | same |
| `cw_pack_str/bin(ctx, ptr, len)` | `cw_pack_str(&mut pc, bytes, len)` |
| `cw_pack_ext/time/insert` | same |
| sticky `pc.return_code` | `pc.return_code` (`0` = ok) |

### Unpack

```rust
use cwpack::{cw_look_ahead, cw_unpack_next, CwUnpackContext, ItemType};

let mut uc = CwUnpackContext::new(packed_bytes);
let _t = cw_look_ahead(&mut uc);
cw_unpack_next(&mut uc);
if uc.return_code == 0 && uc.item.type_code == ItemType::Str as i32 {
    let _s = uc.item_blob();
}
```

Also: `cw_skip_items(&mut uc, count)`.

### Unsafe

**None.** `#![forbid(unsafe_code)]` on the crate. No C ABI / no `extern "C"`.

### Examples

| Example | Purpose |
|---------|---------|
| `examples/rust_bench.rs` | micro-bench |
| `examples/ops_pack.rs` | JSON differential ops → msgpack |
| `examples/fuzz_harness.rs` | self fuzz |
| `examples/cross_write.rs` / `cross_read.rs` | Rust↔C file roundtrip |

## How we prove the port

```bash
cargo test --release          # smoke (C-like API)
make json-diff                # C CWPack vs Rust: identical .mp bytes
make cross-roundtrip          # Rust→file→C verify, C→file→Rust verify
make bench                    # latency / RSS / throughput
make fuzz                     # 60s self-fuzz
```

Original C sources stay available as an **oracle** (sibling `../CWPack` or `CWPACK_SRC`). Hashed reference copy of the upstream module test: `tests/original/` (not linked into Rust).

See [`bench/methodology.md`](bench/methodology.md) and [`docs/API.md`](docs/API.md).

## Layout

```
src/             safe cw_* API + pack/unpack
tests/smoke.rs   Rust API tests
tests/original/  upstream module test kept for reference/hashes
extra-tests/     JSON fixtures + differential harness
fuzz/            fuzz harness
bench/           methodology + results
docs/API.md      API reference
DECISIONS.md     architectural divergences
```

## Scope

**In:** core MessagePack pack/unpack (+ utils semantics where needed in Rust).  
**Out:** C FFI, ObjC/Swift, dump tool, basic-contexts, numeric-extensions.

## License

MIT (same as CWPack).
