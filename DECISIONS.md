# DECISIONS.md — cwpack-rs

Non-trivial divergences from clwi/CWPack and why. (Port Mortem Decision Log.)

1. **C-shaped names, native Rust only**  
   Public API uses `cw_pack_*` / `cw_unpack_*` and context types so C examples nearly copy-paste, but everything is safe Rust — **no C ABI / no `extern "C"`**. Naming similarity ≠ FFI.

2. **Sticky `return_code` on contexts + `Result` at low level**  
   `CwPackContext.return_code` mirrors CWPack sticky errors for familiar call style. Low-level `pack::encode_*` returns `Result` for idiomatic Rust internals.

3. **`#![forbid(unsafe_code)]`**  
   Entire crate is safe. No type-punning; wire endian via `to_be_bytes` / `from_be_bytes`; floats via `to_bits` / `from_bits`.

4. **No link to original `libcwpack` C (Rule §05)**  
   Rust never calls into the C library as implementation. Original CWPack is a **differential oracle** only (`ops_pack_c`, cross-roundtrip, sticky-insert harness, benches).

5. **Equivalence without FFI module test**  
   Instead of linking `cwpack_module_test.c` against a Rust cdylib, we prove behavior via:
   - `make json-diff` — byte-identical MessagePack (JSON→ops→C vs Rust)
   - `make cross-roundtrip` — Rust↔C pack/unpack through `.mp` files + field checks
   - `make sticky-insert` — upstream sticky-insert bug → C decodes wrong `payload=66`; Rust honest error
   - `cargo test` smoke + `make fuzz` self roundtrip  
   Upstream module test is kept hashed under `tests/original/` as reference (not linked).

6. **No separate `cwpack_utils` crate surface**  
   Upstream module test uses typed utils helpers; we exercise the same pack/unpack types through the `cw_*` API and differential harnesses. ObjC/Swift/dump/basic-contexts remain out of scope.

7. **Big-endian via `to_be_bytes` / `from_be_bytes`**  
   Replaced CWPack’s LE/BE type-punning macros. Wire format unchanged.

8. **Object key sort in JSON→ops**  
   Differential JSON harness sorts map keys so C and Rust see the same op stream (isolates codec from JSON parser quirks).

9. **`cw_pack_insert` respects sticky error (upstream latent bug)**  
   CWPack README (“Error handling”) states that after an error, further calls on a context are no-ops — so callers may batch `cw_pack_*` and check `return_code` once at the end, trusting the buffer stays frozen. README (“Backward compatibility”) states EXT is illegal in compatibility mode.

   In stock C (`cwpack.c`), `cw_pack_ext` under `be_compatible` correctly sets sticky `CWP_RC_ILLEGAL_CALL` (-7) and writes nothing. Almost all pack APIs then early-return on non-zero `return_code`. **`cw_pack_insert` omits that check**, so it still runs `cw_pack_reserve_space` + `memcpy` and advances `current` while `return_code` stays `-7`.

   **User-visible corruption** (`make sticky-insert`):

   | Step | Stock C | cwpack-rs |
   |------|---------|-----------|
   | Encode `{status:true, payload:<ext>}` (compat ON) | sticky `-7` after key `payload` (17 bytes) | same |
   | Fallback `insert("BUG!")` | still appends → **21 bytes** | no-op → **17 bytes** |
   | Best-effort send + unpack | `status=true`, **`payload=66`** (`'B'` as fixint) | `status=true`, then **decode error** on payload |

   The receiver does not get a clean failure on C — it gets a **plausible wrong typed value**. That breaks the sticky-error contract that makes delayed checking safe.

   Minimal fix upstream (not applied here — oracle stays stock):

   ```c
   void cw_pack_insert (cw_pack_context* pack_context, const void* v, uint32_t l)
   {
       if (pack_context->return_code)
           return;
       /* ... */
   }
   ```

   cwpack-rs applies the sticky check in `pack_apply` for every `cw_pack_*`, including insert.  
   Repro: `extra-tests/sticky_insert_bug.c` (C exit 1) vs `examples/sticky_insert_ok.rs` (Rust exit 0).  
   Details: [`bench/methodology.md`](bench/methodology.md) §4.

10. **Performance test vs MPack/CMP out of scope**  
    We ship `bench/` comparing C CWPack vs cwpack-rs on a shared workload.

11. **Overflow/underflow handlers omitted in safe API**  
    CWPack function-pointer handlers are for growable buffers/FILE I/O. Safe API uses fixed caller buffers; growth is the caller’s job (`Vec` / resize). Matches “no-alloc core” design.

12. **Timestamp / EXT rules unchanged**  
    Compatibility mode still rejects ext/time (`IllegalCall`); nsec ≥ 1e9 → `ValueError`. Same as C (used as the sticky-error trigger in §9).

13. **Scope cut: no growable malloc contexts**  
    `basic-contexts` stays out; prefer `Vec` if added later.

14. **Docker image builds the Rust library only**  
    Image runs `cargo test`; full C-oracle checks need a sibling `CWPack` checkout (`make json-diff`, `make cross-roundtrip`, `make sticky-insert`, `make bench`).
