# API reference (cwpack-rs)

Companion to the crate README. Behavioral match to [CWPack](https://github.com/clwi/CWPack).

## Preferred API: C names + context class

Types: `CwPackContext`, `CwUnpackContext`  
Functions: `cw_pack_*`, `cw_unpack_next`, `cw_skip_items`, `cw_look_ahead`  
Also available as methods: `pc.cw_pack_map_size(2)`.

```rust
use cwpack::{cw_pack_map_size, cw_pack_str, cw_pack_boolean, cw_pack_unsigned, CwPackContext};

let mut buffer = [0u8; 32];
let mut pc = CwPackContext::new(&mut buffer);
cw_pack_map_size(&mut pc, 2);
cw_pack_str(&mut pc, b"compact", 7);
cw_pack_boolean(&mut pc, true);
cw_pack_str(&mut pc, b"schema", 6);
cw_pack_unsigned(&mut pc, 0);
assert_eq!(pc.return_code, 0);
```

### Pack

| C | Rust |
|---|------|
| `cw_pack_context_init(&pc, buf, len, hpo)` | `CwPackContext::new(&mut buf)` or `cw_pack_context_init(&mut pc, &mut buf)` |
| `cw_pack_set_compatibility` | same |
| `cw_pack_nil/true/false/boolean` | same |
| `cw_pack_signed/unsigned` | `i64` / `u64` |
| `cw_pack_float/double` | same |
| `cw_pack_array_size/map_size` | same |
| `cw_pack_str(ctx, ptr, len)` | `cw_pack_str(&mut pc, bytes, len)` |
| `cw_pack_bin/ext/time/insert` | same shape |
| `pc.return_code` | sticky `i32`, `0` = ok |
| `pc.current - pc.start` | `pc.len_packed()` |

### Unpack

| C | Rust |
|---|------|
| `cw_unpack_context_init` | `CwUnpackContext::new(buf)` |
| `cw_unpack_next` | same; fills `uc.item` (`Decoded`) |
| `cw_skip_items` | same |
| `cw_look_ahead` | same → `i32` type code |
| `uc.item.as.str` | `uc.item_blob()` / `item.blob_off`+`blob_len` |

`Decoded` fields: `type_code`, `boolean`, `u64`/`i64`, `real`/`long_real`, `size`, `blob_off`/`blob_len`, `time_sec`/`time_nsec`.

## Low-level: `pack` / `unpack`

Cursor-style `encode_*` / `unpack_next` returning `Result` — used internally by the C-like layer. See module docs.

## Errors

| Variant | Code |
|---------|------|
| `Ok` | 0 |
| `EndOfInput` | -1 |
| `BufferOverflow` | -2 |
| `BufferUnderflow` | -3 |
| `MalformedInput` | -4 |
| `IllegalCall` | -7 |
| `TypeError` | -10 |
| `ValueError` | -11 |
| `WrongTimestampLength` | -12 |

## C ABI — `ffi` / `utils`

`extern "C"` symbols for linking C tests (`include/cwpack.h`). Not required for normal Rust use.

## Related

- [`README.md`](../README.md)
- [`bench/methodology.md`](../bench/methodology.md)
- [`DECISIONS.md`](../DECISIONS.md)
