//! Safe Rust port of [CWPack](https://github.com/clwi/CWPack) (MessagePack).
//!
//! Fully safe (`forbid(unsafe_code)`). Public API mirrors CWPack C names
//! (`cw_pack_*` / `cw_unpack_*`) on Rust contexts — no C FFI.
//!
//! ```
//! use cwpack::{
//!     cw_pack_boolean, cw_pack_map_size, cw_pack_str, cw_pack_unsigned, CwPackContext,
//! };
//!
//! let mut buffer = [0u8; 32];
//! let mut pc = CwPackContext::new(&mut buffer);
//!
//! cw_pack_map_size(&mut pc, 2);
//! cw_pack_str(&mut pc, b"compact", 7);
//! cw_pack_boolean(&mut pc, true);
//! cw_pack_str(&mut pc, b"schema", 6);
//! cw_pack_unsigned(&mut pc, 0);
//!
//! assert_eq!(pc.return_code, 0);
//! assert_eq!(pc.len_packed(), 18);
//! ```

#![forbid(unsafe_code)]

pub mod cw;
pub mod error;
pub mod item;
pub mod pack;
pub mod unpack;

pub use cw::*;
pub use error::{Error, Result};
pub use item::{Item, ItemType};
