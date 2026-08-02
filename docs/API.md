# API reference (cwpack-rs)

Native safe Rust API with CWPack C names. **No FFI.**

## Public API

Types: `CwPackContext`, `CwUnpackContext`  
Functions: `cw_pack_*`, `cw_unpack_next`, `cw_skip_items`, `cw_look_ahead`  
Methods: `pc.cw_pack_map_size(2)`, …

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
| `cw_pack_context_init(&pc, buf, len, hpo)` | `CwPackContext::new(&mut buf)` |
| `cw_pack_*` | same names; sticky `return_code` |
| `pc.current - pc.start` | `pc.len_packed()` |

### Unpack

| C | Rust |
|---|------|
| `cw_unpack_context_init` | `CwUnpackContext::new(buf)` |
| `cw_unpack_next` | fills `uc.item` (`Decoded`) |
| str/bin payload | `uc.item_blob()` |

## Low-level

`pack::encode_*` / `unpack::unpack_next` — `Result`-based helpers used inside `cw_*`.

## Unsafe

None — `#![forbid(unsafe_code)]`.

## Related

- [`README.md`](../README.md)
- [`bench/methodology.md`](../bench/methodology.md)
- [`DECISIONS.md`](../DECISIONS.md)
