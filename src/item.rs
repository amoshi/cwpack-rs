//! MessagePack item types matching CWPack's `cwpack_item_types`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ItemType {
    Timestamp = -1,
    Nil = 300,
    Boolean = 301,
    PositiveInteger = 302,
    NegativeInteger = 303,
    Float = 304,
    Double = 305,
    Str = 306,
    Bin = 307,
    Array = 308,
    Map = 309,
    Ext = 310,
    NotAnItem = 999,
}

impl ItemType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            -1 => Self::Timestamp,
            300 => Self::Nil,
            301 => Self::Boolean,
            302 => Self::PositiveInteger,
            303 => Self::NegativeInteger,
            304 => Self::Float,
            305 => Self::Double,
            306 => Self::Str,
            307 => Self::Bin,
            308 => Self::Array,
            309 => Self::Map,
            310 => Self::Ext,
            999 => Self::NotAnItem,
            other => {
                // User EXT type codes 0..=127 (and reserved negatives except -1 handled above)
                // are stored as the type discriminant itself in CWPack.
                if (-128..=127).contains(&other) {
                    // SAFETY-adjacent: CWPack reuses the enum discriminant for EXT type tags.
                    // We keep the raw i32 via transmute-free casting for look-ahead.
                }
                // Represent as Ext for unknown; callers that need the raw tag use look_ahead paths.
                Self::Ext
            }
        }
    }
}

/// Owned view of a decoded item (safe Rust API).
#[derive(Clone, Debug, PartialEq)]
pub enum Item<'a> {
    Nil,
    Boolean(bool),
    PositiveInteger(u64),
    NegativeInteger(i64),
    Float(f32),
    Double(f64),
    Str(&'a [u8]),
    Bin(&'a [u8]),
    Array(u32),
    Map(u32),
    Ext { tag: i8, data: &'a [u8] },
    Timestamp { sec: i64, nsec: u32 },
}
