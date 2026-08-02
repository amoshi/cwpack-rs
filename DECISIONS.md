# DECISIONS.md — cwpack-rs

Non-trivial divergences from clwi/CWPack and why. (Port Mortem Decision Log.)

1. **Result vs sticky `return_code`**  
   Idiomatic Rust API uses `error::Error` / `Result`. The C ABI preserves CWPack’s sticky `return_code` field and no-op-after-error semantics so the original module test is unchanged.

2. **Safe core, unsafe only at FFI**  
   Encode/decode live in `pack.rs` / `unpack.rs` with no `unsafe`. Only `ffi.rs` / `utils.rs` dereference C pointers. Rationale: Track A north star + Zero Unsafe bonus eligibility for core logic.

3. **Big-endian via `to_be_bytes` / `from_be_bytes`**  
   Replaced CWPack’s LE/BE type-punning macros and `FORCE_ALIGNMENT` paths with portable byte APIs. Wire format is identical; host endian compile switches are unnecessary.

4. **Floats via `f32::to_bits` / `from_bits`**  
   Avoids C’s `*(uint32_t*)&f` aliasing. Bit-identical MessagePack float/double payloads.

5. **No link to original `libcwpack` C**  
   Contest Rule §05: the port must not FFI into the source library as implementation. The C static library is never a dependency of the Rust crate. Original C is only an optional differential oracle.

6. **C ABI shim for unmodified tests**  
   FAQ/Anatomy allow a thin adapter so hashed C tests call the port. We export symbols matching `cwpack.h` + utils and link `cwpack_module_test.c` against `libcwpack.a` from Rust.

7. **Utils (`goodies/utils`) in P0**  
   Module test `#include`s `cwpack_utils.h`. Omitting utils would fail parity. ObjC/Swift/dump/basic-contexts remain out of scope.

8. **`#[repr(C)]` layouts verified against Clang**  
   Pack=56, Unpack=64, Item=24 on darwin aarch64 — matched before linking. Wrong layout would silent-corrupt the C test.

9. **Overflow/underflow handlers retried in FFI**  
   Safe encoders return `BufferOverflow`; FFI invokes the C handler (if any) and retries. Module test passes NULL handlers — path still matches CWPack error codes.

10. **EXT type tags stored as `item.type_` discriminant**  
    CWPack overloads `cwpack_item_types` with raw EXT tags (−128…127). We preserve that for look-ahead / unpack parity instead of a separate tag field in the C union.

11. **`cw_pack_insert` early-exit on sticky error**  
    C’s `cw_pack_insert` does not check `return_code` first; our FFI `pack_fn` does. Documented divergence; unused by module test. Could be aligned later if needed.

12. **Performance test vs MPack/CMP out of scope**  
    Requires sibling checkouts of other repos. We ship our own `bench/` comparing C CWPack vs cwpack-rs instead of the upstream comparative harness.

13. **NaN-as-0 in utils**  
    CWPack defines `#define NaN 0` for typed float unpackers. We return `0.0` on error the same way (not IEEE NaN) for ABI parity.

14. **Scope cut: no growable malloc contexts in v0.1**  
    `basic-contexts` uses `malloc`/`fwrite`. Not required for module-test parity. Prefer `Vec` if ported later (idiomatic Rust) — would be another DECISIONS entry.
