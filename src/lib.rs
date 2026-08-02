//! Safe Rust port of [CWPack](https://github.com/clwi/CWPack) (MessagePack).
//!
//! # Quick start
//!
//! ```
//! use cwpack::pack;
//!
//! let mut buf = [0u8; 32];
//! let end = buf.len();
//! let mut pos = 0;
//! pack::encode_map_size(&mut buf, &mut pos, end, 2).unwrap();
//! pack::encode_str(&mut buf, &mut pos, end, b"compact", false).unwrap();
//! pack::encode_bool(&mut buf, &mut pos, end, true).unwrap();
//! pack::encode_str(&mut buf, &mut pos, end, b"schema", false).unwrap();
//! pack::encode_unsigned(&mut buf, &mut pos, end, 0).unwrap();
//! assert_eq!(pos, 18);
//! ```
//!
//! Unpack with [`unpack::unpack_next`]; skip/look-ahead via [`unpack::skip_items`] /
//! [`unpack::look_ahead`]. Errors use [`Error`] / [`Result`] (C `CWP_RC_*` codes).
//!
//! C ABI: [`ffi`], [`utils`] (for linking original C tests). See the repo `README.md`
//! for full API notes.

#![allow(clippy::missing_safety_doc)]

pub mod error;
pub mod ffi;
pub mod item;
pub mod pack;
pub mod unpack;
pub mod utils;

pub use error::{Error, Result};
pub use item::{Item, ItemType};
