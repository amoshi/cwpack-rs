//! Safe Rust port of [CWPack](https://github.com/clwi/CWPack) (MessagePack).
//!
//! - Idiomatic safe API: [`error`], [`pack`], [`unpack`]
//! - C ABI for original test suite: [`ffi`], [`utils`]

#![allow(clippy::missing_safety_doc)]

pub mod error;
pub mod ffi;
pub mod item;
pub mod pack;
pub mod unpack;
pub mod utils;

pub use error::{Error, Result};
pub use item::{Item, ItemType};
