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

## Run original module test against the port

```bash
chmod +x tests/original/run_against_rust.sh
./tests/original/run_against_rust.sh
```

Expected: `CWPack module test completed, no errors detected`

Original test file hashes: `tests/original/SHA256SUMS`.

## Layout

```
src/           safe pack/unpack + C ABI (ffi/utils)
include/       original C headers (for linking the C test)
tests/original/  unmodified cwpack_module_test.c
tests/port/    Rust-side tests (optional)
fuzz/          differential harness
bench/         methodology + results
DECISIONS.md   architectural divergences
```

## Scope

**In:** core `cwpack.c` API + `goodies/utils` (required by module test).  
**Out (documented):** ObjC/Swift bindings, dump tool, basic-contexts, numeric-extensions, MPack/CMP perf comparison binary.

## License

MIT (same as CWPack).
