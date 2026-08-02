# DECISIONS.md — cwpack-rs

Non-trivial divergences from clwi/CWPack and why. (Port Mortem Decision Log.)

1. **C-shaped names, native Rust only**  
   Public API uses `cw_pack_*` / `cw_unpack_*` and context types so C examples nearly copy-paste, but everything is safe Rust — **no C ABI / no `extern "C"`**. Naming similarity ≠ FFI.

2. **Sticky `return_code` on contexts + `Result` at low level**  
   `CwPackContext.return_code` mirrors CWPack sticky errors for familiar call style. Low-level `pack::encode_*` returns `Result` for idiomatic Rust internals.

3. **`#![forbid(unsafe_code)]`**  
   Entire crate is safe. No type-punning; wire endian via `to_be_bytes` / `from_be_bytes`; floats via `to_bits` / `from_bits`.

4. **No link to original `libcwpack` C (Rule §05)**  
   Rust never calls into the C library as implementation. Original CWPack is a **differential oracle** only (`ops_pack_c` / benches).

5. **Equivalence without FFI module test**  
   Instead of linking `cwpack_module_test.c` against a Rust cdylib, we prove byte-identity via `extra-tests` JSON→ops→MessagePack (C vs Rust) plus smoke tests and fuzz. Upstream test file is kept hashed under `tests/original/` as reference.

6. **Utils typed helpers not exported as C**  
   Upstream module test uses `cwpack_utils`; we cover the same pack/unpack surface in Rust. ObjC/Swift/dump/basic-contexts remain out of scope.

7. **Big-endian via `to_be_bytes` / `from_be_bytes`**  
   Replaced CWPack’s LE/BE type-punning macros. Wire format unchanged.

8. **Object key sort in JSON→ops**  
   Differential JSON harness sorts map keys so C and Rust see the same op stream (isolates codec from JSON parser quirks).

9. **`cw_pack_insert` respects sticky error**  
   C’s insert does not check `return_code` first; our `cw_pack_*` layer no-ops when sticky error is set. Documented; unused by smoke path.

10. **Performance test vs MPack/CMP out of scope**  
    We ship `bench/` comparing C CWPack vs cwpack-rs on a shared workload.

11. **Overflow/underflow handlers omitted in safe API**  
    CWPack function-pointer handlers are for growable buffers/FILE I/O. Safe API uses fixed caller buffers; growth is the caller’s job (`Vec` / resize). Matches “no-alloc core” design.

12. **Timestamp / EXT rules unchanged**  
    Compatibility mode still rejects ext/time; nsec ≥ 1e9 → `ValueError`.

13. **Scope cut: no growable malloc contexts**  
    `basic-contexts` stays out; prefer `Vec` if added later.

14. **Docker image builds the Rust library only**  
    No C-linked module test in the image; run `cargo test` / `make json-diff` for verification.
