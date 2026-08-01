//! Safe MessagePack unpacking (behavioral match to CWPack).

use crate::error::{Error, Result};
use crate::item::ItemType;

#[derive(Clone, Debug)]
pub struct Decoded {
    pub type_code: i32,
    pub boolean: bool,
    pub u64: u64,
    pub i64: i64,
    pub real: f32,
    pub long_real: f64,
    pub size: u32,
    pub blob_off: usize,
    pub blob_len: u32,
    pub time_sec: i64,
    pub time_nsec: u32,
}

impl Decoded {
    fn typ(t: ItemType) -> Self {
        Self {
            type_code: t as i32,
            boolean: false,
            u64: 0,
            i64: 0,
            real: 0.0,
            long_real: 0.0,
            size: 0,
            blob_off: 0,
            blob_len: 0,
            time_sec: 0,
            time_nsec: 0,
        }
    }
}

fn load16(buf: &[u8]) -> u16 {
    u16::from_be_bytes([buf[0], buf[1]])
}
fn load32(buf: &[u8]) -> u32 {
    u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]])
}
fn load64(buf: &[u8]) -> u64 {
    u64::from_be_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    ])
}

/// Ensure `need` bytes available from `pos` up to `end`. Updates `pos` to after those bytes.
/// Returns offset where the reserved region started.
fn assert_space(pos: &mut usize, end: usize, need: usize, at_header: bool) -> Result<usize> {
    let start = *pos;
    let nyp = start.checked_add(need).ok_or(if at_header {
        Error::EndOfInput
    } else {
        Error::BufferUnderflow
    })?;
    if nyp > end {
        return Err(if at_header {
            Error::EndOfInput
        } else {
            Error::BufferUnderflow
        });
    }
    *pos = nyp;
    Ok(start)
}

pub fn unpack_next(buf: &[u8], pos: &mut usize, end: usize) -> Result<Decoded> {
    let p0 = assert_space(pos, end, 1, true)?;
    let c = buf[p0];
    match c {
        0x00..=0x7f => {
            let mut d = Decoded::typ(ItemType::PositiveInteger);
            d.i64 = c as i64;
            d.u64 = c as u64;
            Ok(d)
        }
        0x80..=0x8f => {
            let mut d = Decoded::typ(ItemType::Map);
            d.size = (c & 0x0f) as u32;
            Ok(d)
        }
        0x90..=0x9f => {
            let mut d = Decoded::typ(ItemType::Array);
            d.size = (c & 0x0f) as u32;
            Ok(d)
        }
        0xa0..=0xbf => {
            let len = (c & 0x1f) as u32;
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Str);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xc0 => Ok(Decoded::typ(ItemType::Nil)),
        0xc2 => {
            let mut d = Decoded::typ(ItemType::Boolean);
            d.boolean = false;
            Ok(d)
        }
        0xc3 => {
            let mut d = Decoded::typ(ItemType::Boolean);
            d.boolean = true;
            Ok(d)
        }
        0xc4 => {
            let p = assert_space(pos, end, 1, false)?;
            let len = buf[p] as u32;
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Bin);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xc5 => {
            let p = assert_space(pos, end, 2, false)?;
            let len = load16(&buf[p..]) as u32;
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Bin);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xc6 => {
            let p = assert_space(pos, end, 4, false)?;
            let len = load32(&buf[p..]);
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Bin);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xc7 => {
            let p = assert_space(pos, end, 1, false)?;
            let len = buf[p] as u32;
            let tp = assert_space(pos, end, 1, false)?;
            let tag = buf[tp] as i8;
            if tag as i32 == ItemType::Timestamp as i32 {
                if len == 12 {
                    let p4 = assert_space(pos, end, 4, false)?;
                    let nsec = load32(&buf[p4..]);
                    let p8 = assert_space(pos, end, 8, false)?;
                    let sec = load64(&buf[p8..]) as i64;
                    let mut d = Decoded::typ(ItemType::Timestamp);
                    d.time_sec = sec;
                    d.time_nsec = nsec;
                    return Ok(d);
                }
                return Err(Error::WrongTimestampLength);
            }
            let off = assert_space(pos, end, len as usize, false)?;
            let d = Decoded {
                type_code: tag as i32,
                boolean: false,
                u64: 0,
                i64: 0,
                real: 0.0,
                long_real: 0.0,
                size: 0,
                blob_off: off,
                blob_len: len,
                time_sec: 0,
                time_nsec: 0,
            };
            // EXT payload in blob fields; type_code is the EXT tag.
            let _ = ItemType::Ext;
            Ok(d)
        }
        0xc8 => {
            let p = assert_space(pos, end, 2, false)?;
            let len = load16(&buf[p..]) as u32;
            let tp = assert_space(pos, end, 1, false)?;
            let tag = buf[tp] as i8;
            let off = assert_space(pos, end, len as usize, false)?;
            Ok(Decoded {
                type_code: tag as i32,
                boolean: false,
                u64: 0,
                i64: 0,
                real: 0.0,
                long_real: 0.0,
                size: 0,
                blob_off: off,
                blob_len: len,
                time_sec: 0,
                time_nsec: 0,
            })
        }
        0xc9 => {
            let p = assert_space(pos, end, 4, false)?;
            let len = load32(&buf[p..]);
            let tp = assert_space(pos, end, 1, false)?;
            let tag = buf[tp] as i8;
            let off = assert_space(pos, end, len as usize, false)?;
            Ok(Decoded {
                type_code: tag as i32,
                boolean: false,
                u64: 0,
                i64: 0,
                real: 0.0,
                long_real: 0.0,
                size: 0,
                blob_off: off,
                blob_len: len,
                time_sec: 0,
                time_nsec: 0,
            })
        }
        0xca => {
            let p = assert_space(pos, end, 4, false)?;
            let bits = load32(&buf[p..]);
            let mut d = Decoded::typ(ItemType::Float);
            d.real = f32::from_bits(bits);
            Ok(d)
        }
        0xcb => {
            let p = assert_space(pos, end, 8, false)?;
            let bits = load64(&buf[p..]);
            let mut d = Decoded::typ(ItemType::Double);
            d.u64 = bits;
            d.long_real = f64::from_bits(bits);
            Ok(d)
        }
        0xcc => {
            let p = assert_space(pos, end, 1, false)?;
            let mut d = Decoded::typ(ItemType::PositiveInteger);
            d.u64 = buf[p] as u64;
            d.i64 = buf[p] as i64;
            Ok(d)
        }
        0xcd => {
            let p = assert_space(pos, end, 2, false)?;
            let v = load16(&buf[p..]) as u64;
            let mut d = Decoded::typ(ItemType::PositiveInteger);
            d.u64 = v;
            d.i64 = v as i64;
            Ok(d)
        }
        0xce => {
            let p = assert_space(pos, end, 4, false)?;
            let v = load32(&buf[p..]) as u64;
            let mut d = Decoded::typ(ItemType::PositiveInteger);
            d.u64 = v;
            d.i64 = v as i64;
            Ok(d)
        }
        0xcf => {
            let p = assert_space(pos, end, 8, false)?;
            let v = load64(&buf[p..]);
            let mut d = Decoded::typ(ItemType::PositiveInteger);
            d.u64 = v;
            d.i64 = v as i64;
            Ok(d)
        }
        0xd0 => {
            let p = assert_space(pos, end, 1, false)?;
            let v = buf[p] as i8 as i64;
            let mut d = Decoded::typ(ItemType::NegativeInteger);
            d.i64 = v;
            d.u64 = v as u64;
            if v >= 0 {
                d.type_code = ItemType::PositiveInteger as i32;
            }
            Ok(d)
        }
        0xd1 => {
            let p = assert_space(pos, end, 2, false)?;
            let v = load16(&buf[p..]) as i16 as i64;
            let mut d = Decoded::typ(ItemType::NegativeInteger);
            d.i64 = v;
            d.u64 = v as u64;
            if v >= 0 {
                d.type_code = ItemType::PositiveInteger as i32;
            }
            Ok(d)
        }
        0xd2 => {
            let p = assert_space(pos, end, 4, false)?;
            let v = load32(&buf[p..]) as i32 as i64;
            let mut d = Decoded::typ(ItemType::NegativeInteger);
            d.i64 = v;
            d.u64 = v as u64;
            if v >= 0 {
                d.type_code = ItemType::PositiveInteger as i32;
            }
            Ok(d)
        }
        0xd3 => {
            let p = assert_space(pos, end, 8, false)?;
            let v = load64(&buf[p..]) as i64;
            let mut d = Decoded::typ(ItemType::NegativeInteger);
            d.i64 = v;
            d.u64 = v as u64;
            if v >= 0 {
                d.type_code = ItemType::PositiveInteger as i32;
            }
            Ok(d)
        }
        0xd4 => decode_fixext(buf, pos, end, 1),
        0xd5 => decode_fixext(buf, pos, end, 2),
        0xd6 => decode_fixext(buf, pos, end, 4),
        0xd7 => decode_fixext(buf, pos, end, 8),
        0xd8 => decode_fixext(buf, pos, end, 16),
        0xd9 => {
            let p = assert_space(pos, end, 1, false)?;
            let len = buf[p] as u32;
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Str);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xda => {
            let p = assert_space(pos, end, 2, false)?;
            let len = load16(&buf[p..]) as u32;
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Str);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xdb => {
            let p = assert_space(pos, end, 4, false)?;
            let len = load32(&buf[p..]);
            let off = assert_space(pos, end, len as usize, false)?;
            let mut d = Decoded::typ(ItemType::Str);
            d.blob_len = len;
            d.blob_off = off;
            Ok(d)
        }
        0xdc => {
            let p = assert_space(pos, end, 2, false)?;
            let mut d = Decoded::typ(ItemType::Array);
            d.size = load16(&buf[p..]) as u32;
            Ok(d)
        }
        0xdd => {
            let p = assert_space(pos, end, 4, false)?;
            let mut d = Decoded::typ(ItemType::Array);
            d.size = load32(&buf[p..]);
            Ok(d)
        }
        0xde => {
            let p = assert_space(pos, end, 2, false)?;
            let mut d = Decoded::typ(ItemType::Map);
            d.size = load16(&buf[p..]) as u32;
            Ok(d)
        }
        0xdf => {
            let p = assert_space(pos, end, 4, false)?;
            let mut d = Decoded::typ(ItemType::Map);
            d.size = load32(&buf[p..]);
            Ok(d)
        }
        0xe0..=0xff => {
            let mut d = Decoded::typ(ItemType::NegativeInteger);
            d.i64 = c as i8 as i64;
            d.u64 = d.i64 as u64;
            Ok(d)
        }
        0xc1 => Err(Error::MalformedInput),
    }
}

fn decode_fixext(buf: &[u8], pos: &mut usize, end: usize, len: u32) -> Result<Decoded> {
    let p = assert_space(pos, end, (len + 1) as usize, false)?;
    let tag = buf[p] as i8;
    let data_off = p + 1;
    if tag as i32 == ItemType::Timestamp as i32 {
        if len == 4 {
            let sec = load32(&buf[data_off..]) as i64;
            let mut d = Decoded::typ(ItemType::Timestamp);
            d.time_sec = sec;
            d.time_nsec = 0;
            return Ok(d);
        } else if len == 8 {
            let data64 = load64(&buf[data_off..]);
            let mut d = Decoded::typ(ItemType::Timestamp);
            d.time_sec = (data64 & 0x00000003ffffffff) as i64;
            d.time_nsec = (data64 >> 34) as u32;
            return Ok(d);
        } else {
            return Err(Error::WrongTimestampLength);
        }
    }
    Ok(Decoded {
        type_code: tag as i32,
        boolean: false,
        u64: 0,
        i64: 0,
        real: 0.0,
        long_real: 0.0,
        size: 0,
        blob_off: data_off,
        blob_len: len,
        time_sec: 0,
        time_nsec: 0,
    })
}

pub fn skip_items(buf: &[u8], pos: &mut usize, end: usize, mut item_count: i64) -> Result<()> {
    while item_count > 0 {
        item_count -= 1;
        let p0 = assert_space(pos, end, 1, true)?;
        let c = buf[p0];
        match c {
            0x00..=0x7f | 0xe0..=0xff | 0xc0 | 0xc2 | 0xc3 => {}
            0xcc | 0xd0 => {
                assert_space(pos, end, 1, false)?;
            }
            0xcd | 0xd1 | 0xd4 => {
                assert_space(pos, end, 2, false)?;
            }
            0xd5 => {
                assert_space(pos, end, 3, false)?;
            }
            0xca | 0xce | 0xd2 => {
                assert_space(pos, end, 4, false)?;
            }
            0xd6 => {
                assert_space(pos, end, 5, false)?;
            }
            0xcb | 0xcf | 0xd3 => {
                assert_space(pos, end, 8, false)?;
            }
            0xd7 => {
                assert_space(pos, end, 9, false)?;
            }
            0xd8 => {
                assert_space(pos, end, 17, false)?;
            }
            0xa0..=0xbf => {
                assert_space(pos, end, (c & 0x1f) as usize, false)?;
            }
            0xd9 | 0xc4 => {
                let p = assert_space(pos, end, 1, false)?;
                let n = buf[p] as usize;
                assert_space(pos, end, n, false)?;
            }
            0xda | 0xc5 => {
                let p = assert_space(pos, end, 2, false)?;
                let n = load16(&buf[p..]) as usize;
                assert_space(pos, end, n, false)?;
            }
            0xdb | 0xc6 => {
                let p = assert_space(pos, end, 4, false)?;
                let n = load32(&buf[p..]) as usize;
                assert_space(pos, end, n, false)?;
            }
            0x80..=0x8f => {
                item_count += 2 * (c & 15) as i64;
            }
            0x90..=0x9f => {
                item_count += (c & 15) as i64;
            }
            0xdc => {
                let p = assert_space(pos, end, 2, false)?;
                item_count += load16(&buf[p..]) as i64;
            }
            0xde => {
                let p = assert_space(pos, end, 2, false)?;
                item_count += 2 * load16(&buf[p..]) as i64;
            }
            0xdd => {
                let p = assert_space(pos, end, 4, false)?;
                item_count += load32(&buf[p..]) as i64;
            }
            0xdf => {
                let p = assert_space(pos, end, 4, false)?;
                item_count += 2 * load32(&buf[p..]) as i64;
            }
            0xc7 => {
                let p = assert_space(pos, end, 1, false)?;
                let n = buf[p] as usize;
                assert_space(pos, end, n + 1, false)?;
            }
            0xc8 => {
                let p = assert_space(pos, end, 2, false)?;
                let n = load16(&buf[p..]) as usize;
                assert_space(pos, end, n + 1, false)?;
            }
            0xc9 => {
                let p = assert_space(pos, end, 4, false)?;
                let n = load32(&buf[p..]) as usize;
                assert_space(pos, end, n + 1, false)?;
            }
            _ => return Err(Error::MalformedInput),
        }
    }
    Ok(())
}

pub fn look_ahead(buf: &[u8], pos: usize, end: usize) -> Result<i32> {
    let mut p = pos;
    let p0 = assert_space(&mut p, end, 1, true)?;
    let c = buf[p0];
    // step back like C: current -= 1 after assert_space advanced
    // Caller keeps pos unchanged; we used local p.
    match c {
        0x00..=0x7f => Ok(ItemType::PositiveInteger as i32),
        0x80..=0x8f => Ok(ItemType::Map as i32),
        0x90..=0x9f => Ok(ItemType::Array as i32),
        0xa0..=0xbf => Ok(ItemType::Str as i32),
        0xc0 => Ok(ItemType::Nil as i32),
        0xc2 | 0xc3 => Ok(ItemType::Boolean as i32),
        0xc4..=0xc6 => Ok(ItemType::Bin as i32),
        0xc7 => {
            let mut q = pos;
            assert_space(&mut q, end, 3, true)?;
            let tag = buf[pos + 2] as i8 as i32;
            if tag == ItemType::Timestamp as i32 {
                Ok(ItemType::Timestamp as i32)
            } else {
                Ok(tag)
            }
        }
        0xc8 => {
            let mut q = pos;
            assert_space(&mut q, end, 4, true)?;
            Ok(buf[pos + 3] as i8 as i32)
        }
        0xc9 => {
            let mut q = pos;
            assert_space(&mut q, end, 6, true)?;
            Ok(buf[pos + 5] as i8 as i32)
        }
        0xca => Ok(ItemType::Float as i32),
        0xcb => Ok(ItemType::Double as i32),
        0xcc..=0xcf => Ok(ItemType::PositiveInteger as i32),
        0xd0..=0xd3 => Ok(ItemType::NegativeInteger as i32),
        0xd4..=0xd8 => {
            let mut q = pos;
            assert_space(&mut q, end, 2, true)?;
            Ok(buf[pos + 1] as i8 as i32)
        }
        0xd9..=0xdb => Ok(ItemType::Str as i32),
        0xdc | 0xdd => Ok(ItemType::Array as i32),
        0xde | 0xdf => Ok(ItemType::Map as i32),
        0xe0..=0xff => Ok(ItemType::NegativeInteger as i32),
        _ => Ok(ItemType::NotAnItem as i32),
    }
}
