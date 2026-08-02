# API reference (cwpack-rs)

Companion to the crate README. Behavioral match to [CWPack](https://github.com/clwi/CWPack).

## Design

- **No-alloc core:** caller supplies `&mut [u8]` (pack) or `&[u8]` (unpack).
- **Cursor:** `pos` is the next byte to write/read; `end` is the exclusive limit (`buf.len()` for a full buffer).
- **Containers:** pack/unpack size header first, then elements one-by-one (map: `2 * size` items).
- **Compatibility mode:** `be_compatible == true` disables str8, maps bin→str, rejects ext/time (same as CWPack).

## Safe pack — `cwpack::pack`

| Function | Signature (conceptually) | Notes |
|----------|--------------------------|-------|
| `encode_nil` | `(buf, pos, end) -> Result<()>` | `0xc0` |
| `encode_bool` | `(…, bool)` | `0xc2` / `0xc3` |
| `encode_unsigned` | `(…, u64)` | positive int family |
| `encode_signed` | `(…, i64)` | negative / mixed |
| `encode_float` / `encode_double` | `(…, f32/f64)` | IEEE bits on wire |
| `encode_str` | `(…, &[u8], be_compatible)` | UTF-8 not validated |
| `encode_bin` | `(…, &[u8], be_compatible)` | |
| `encode_ext` | `(…, i8, &[u8], be_compatible)` | |
| `encode_array_size` / `encode_map_size` | `(…, u32)` | then pack elements |
| `encode_time` | `(…, sec: i64, nsec: u32, be_compatible)` | ext type −1 |
| `encode_insert` | `(…, &[u8])` | raw splice |
| `reserve` | low-level slice reservation | used internally |

On success `pos` advances. On failure returns `Error::BufferOverflow` (or `IllegalCall` / `ValueError` for ext/time rules).

## Safe unpack — `cwpack::unpack`

| Function | Notes |
|----------|-------|
| `unpack_next(buf, pos, end) -> Result<Decoded>` | consumes one item |
| `skip_items(buf, pos, end, count)` | like CWPack `cw_skip_items` |
| `look_ahead(buf, pos, end) -> Result<i32>` | type code, does not advance `pos` |

### `Decoded`

| Field | Meaning |
|-------|---------|
| `type_code` | CWPack `cwpack_item_types` / EXT tag |
| `boolean` | bool payload |
| `u64` / `i64` | integer payload (union overlay in C) |
| `real` / `long_real` | f32 / f64 |
| `size` | array/map length |
| `blob_off` / `blob_len` | str/bin/ext bytes inside `buf` |
| `time_sec` / `time_nsec` | timestamp |

Use `cwpack::ItemType` for named constants (`Nil = 300`, `Str = 306`, …; timestamp = `-1`).

## Errors — `cwpack::Error`

| Variant | Code | Typical cause |
|---------|------|----------------|
| `Ok` | 0 | |
| `EndOfInput` | -1 | unpack past end (header) |
| `BufferOverflow` | -2 | pack past `end` |
| `BufferUnderflow` | -3 | unpack past end (body) |
| `MalformedInput` | -4 | bad tag |
| `IllegalCall` | -7 | ext/time in compatible mode |
| `TypeError` | -10 | utils typed unpack mismatch |
| `ValueError` | -11 | e.g. nsec ≥ 1e9 |
| `WrongTimestampLength` | -12 | |

`Result<T>` is `core::result::Result<T, Error>`.

## C ABI — `cwpack::ffi` / `utils`

Exported `extern "C"` names match `include/cwpack.h` and utils headers. Context structs are `#[repr(C)]` (pack 56 / unpack 64 / item 24 on darwin arm64). Sticky `return_code`: after an error, pack/unpack no-ops until re-init.

Link example: see `run-module-test.sh`.

## Related docs

- [`README.md`](../README.md) — quick start
- [`bench/methodology.md`](../bench/methodology.md) — benches + JSON differential
- [`DECISIONS.md`](../DECISIONS.md) — divergences from C
