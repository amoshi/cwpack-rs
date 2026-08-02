//! Safe Rust port of [CWPack](https://github.com/clwi/CWPack) (MessagePack).
//!
//! # C-like API (preferred for copy-paste from CWPack)
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
//!
//! Same calls as methods: `pc.cw_pack_map_size(2)`.
//!
//! Low-level buffer helpers: [`pack`], [`unpack`]. C ABI for tests: [`ffi`], [`utils`].

#![allow(clippy::missing_safety_doc)]

pub mod cw;
pub mod error;
pub mod ffi;
pub mod item;
pub mod pack;
pub mod unpack;
pub mod utils;

pub use cw::*;
pub use error::{Error, Result};
pub use item::{Item, ItemType};
