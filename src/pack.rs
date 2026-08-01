//! Safe MessagePack packing helpers (big-endian wire format).

use crate::error::{Error, Result};

/// Write `need` bytes at `pos`, returning a mutable subslice and updating `pos`.
pub fn reserve<'a>(
    buf: &'a mut [u8],
    pos: &mut usize,
    end: usize,
    need: usize,
) -> Result<&'a mut [u8]> {
    let start = *pos;
    let nyp = start.checked_add(need).ok_or(Error::BufferOverflow)?;
    if nyp > end {
        return Err(Error::BufferOverflow);
    }
    *pos = nyp;
    Ok(&mut buf[start..nyp])
}

pub fn pack_u8(slot: &mut [u8], tag: u8, v: u8) {
    slot[0] = tag;
    slot[1] = v;
}

pub fn pack_u16(slot: &mut [u8], tag: u8, v: u16) {
    slot[0] = tag;
    slot[1..3].copy_from_slice(&v.to_be_bytes());
}

pub fn pack_u32(slot: &mut [u8], tag: u8, v: u32) {
    slot[0] = tag;
    slot[1..5].copy_from_slice(&v.to_be_bytes());
}

pub fn pack_u64(slot: &mut [u8], tag: u8, v: u64) {
    slot[0] = tag;
    slot[1..9].copy_from_slice(&v.to_be_bytes());
}

pub fn encode_unsigned(buf: &mut [u8], pos: &mut usize, end: usize, i: u64) -> Result<()> {
    if i < 128 {
        let s = reserve(buf, pos, end, 1)?;
        s[0] = i as u8;
        return Ok(());
    }
    if i < 256 {
        let s = reserve(buf, pos, end, 2)?;
        pack_u8(s, 0xcc, i as u8);
        return Ok(());
    }
    if i < 0x1_0000 {
        let s = reserve(buf, pos, end, 3)?;
        pack_u16(s, 0xcd, i as u16);
        return Ok(());
    }
    if i < 0x1_0000_0000 {
        let s = reserve(buf, pos, end, 5)?;
        pack_u32(s, 0xce, i as u32);
        return Ok(());
    }
    let s = reserve(buf, pos, end, 9)?;
    pack_u64(s, 0xcf, i);
    Ok(())
}

pub fn encode_signed(buf: &mut [u8], pos: &mut usize, end: usize, i: i64) -> Result<()> {
    if i > 127 {
        return encode_unsigned(buf, pos, end, i as u64);
    }
    if i >= -32 {
        let s = reserve(buf, pos, end, 1)?;
        s[0] = i as u8;
        return Ok(());
    }
    if i >= -128 {
        let s = reserve(buf, pos, end, 2)?;
        pack_u8(s, 0xd0, i as u8);
        return Ok(());
    }
    if i >= -32768 {
        let s = reserve(buf, pos, end, 3)?;
        pack_u16(s, 0xd1, i as u16);
        return Ok(());
    }
    if i >= (0xffffffff80000000u64 as i64) {
        let s = reserve(buf, pos, end, 5)?;
        pack_u32(s, 0xd2, i as u32);
        return Ok(());
    }
    let s = reserve(buf, pos, end, 9)?;
    pack_u64(s, 0xd3, i as u64);
    Ok(())
}

pub fn encode_float(buf: &mut [u8], pos: &mut usize, end: usize, f: f32) -> Result<()> {
    let s = reserve(buf, pos, end, 5)?;
    pack_u32(s, 0xca, f.to_bits());
    Ok(())
}

pub fn encode_double(buf: &mut [u8], pos: &mut usize, end: usize, d: f64) -> Result<()> {
    let s = reserve(buf, pos, end, 9)?;
    pack_u64(s, 0xcb, d.to_bits());
    Ok(())
}

pub fn encode_nil(buf: &mut [u8], pos: &mut usize, end: usize) -> Result<()> {
    let s = reserve(buf, pos, end, 1)?;
    s[0] = 0xc0;
    Ok(())
}

pub fn encode_bool(buf: &mut [u8], pos: &mut usize, end: usize, b: bool) -> Result<()> {
    let s = reserve(buf, pos, end, 1)?;
    s[0] = if b { 0xc3 } else { 0xc2 };
    Ok(())
}

pub fn encode_array_size(buf: &mut [u8], pos: &mut usize, end: usize, n: u32) -> Result<()> {
    if n < 16 {
        let s = reserve(buf, pos, end, 1)?;
        s[0] = 0x90 | (n as u8);
        return Ok(());
    }
    if n < 65536 {
        let s = reserve(buf, pos, end, 3)?;
        pack_u16(s, 0xdc, n as u16);
        return Ok(());
    }
    let s = reserve(buf, pos, end, 5)?;
    pack_u32(s, 0xdd, n);
    Ok(())
}

pub fn encode_map_size(buf: &mut [u8], pos: &mut usize, end: usize, n: u32) -> Result<()> {
    if n < 16 {
        let s = reserve(buf, pos, end, 1)?;
        s[0] = 0x80 | (n as u8);
        return Ok(());
    }
    if n < 65536 {
        let s = reserve(buf, pos, end, 3)?;
        pack_u16(s, 0xde, n as u16);
        return Ok(());
    }
    let s = reserve(buf, pos, end, 5)?;
    pack_u32(s, 0xdf, n);
    Ok(())
}

pub fn encode_str(
    buf: &mut [u8],
    pos: &mut usize,
    end: usize,
    v: &[u8],
    be_compatible: bool,
) -> Result<()> {
    let l = v.len() as u32;
    if l < 32 {
        let s = reserve(buf, pos, end, (l + 1) as usize)?;
        s[0] = 0xa0 + (l as u8);
        s[1..].copy_from_slice(v);
        return Ok(());
    }
    if l < 256 && !be_compatible {
        let s = reserve(buf, pos, end, (l + 2) as usize)?;
        s[0] = 0xd9;
        s[1] = l as u8;
        s[2..].copy_from_slice(v);
        return Ok(());
    }
    if l < 65536 {
        let s = reserve(buf, pos, end, (l + 3) as usize)?;
        s[0] = 0xda;
        s[1..3].copy_from_slice(&(l as u16).to_be_bytes());
        s[3..].copy_from_slice(v);
        return Ok(());
    }
    let s = reserve(buf, pos, end, (l + 5) as usize)?;
    s[0] = 0xdb;
    s[1..5].copy_from_slice(&l.to_be_bytes());
    s[5..].copy_from_slice(v);
    Ok(())
}

pub fn encode_bin(
    buf: &mut [u8],
    pos: &mut usize,
    end: usize,
    v: &[u8],
    be_compatible: bool,
) -> Result<()> {
    if be_compatible {
        return encode_str(buf, pos, end, v, true);
    }
    let l = v.len() as u32;
    if l < 256 {
        let s = reserve(buf, pos, end, (l + 2) as usize)?;
        s[0] = 0xc4;
        s[1] = l as u8;
        s[2..].copy_from_slice(v);
        return Ok(());
    }
    if l < 65536 {
        let s = reserve(buf, pos, end, (l + 3) as usize)?;
        s[0] = 0xc5;
        s[1..3].copy_from_slice(&(l as u16).to_be_bytes());
        s[3..].copy_from_slice(v);
        return Ok(());
    }
    let s = reserve(buf, pos, end, (l + 5) as usize)?;
    s[0] = 0xc6;
    s[1..5].copy_from_slice(&l.to_be_bytes());
    s[5..].copy_from_slice(v);
    Ok(())
}

pub fn encode_ext(
    buf: &mut [u8],
    pos: &mut usize,
    end: usize,
    typ: i8,
    v: &[u8],
    be_compatible: bool,
) -> Result<()> {
    if be_compatible {
        return Err(Error::IllegalCall);
    }
    let l = v.len() as u32;
    let typ_u = typ as u8;
    match l {
        1 => {
            let s = reserve(buf, pos, end, 3)?;
            s[0] = 0xd4;
            s[1] = typ_u;
            s[2] = v[0];
            Ok(())
        }
        2 => {
            let s = reserve(buf, pos, end, 4)?;
            s[0] = 0xd5;
            s[1] = typ_u;
            s[2..].copy_from_slice(v);
            Ok(())
        }
        4 => {
            let s = reserve(buf, pos, end, 6)?;
            s[0] = 0xd6;
            s[1] = typ_u;
            s[2..].copy_from_slice(v);
            Ok(())
        }
        8 => {
            let s = reserve(buf, pos, end, 10)?;
            s[0] = 0xd7;
            s[1] = typ_u;
            s[2..].copy_from_slice(v);
            Ok(())
        }
        16 => {
            let s = reserve(buf, pos, end, 18)?;
            s[0] = 0xd8;
            s[1] = typ_u;
            s[2..].copy_from_slice(v);
            Ok(())
        }
        _ if l < 256 => {
            let s = reserve(buf, pos, end, (l + 3) as usize)?;
            s[0] = 0xc7;
            s[1] = l as u8;
            s[2] = typ_u;
            s[3..].copy_from_slice(v);
            Ok(())
        }
        _ if l < 65536 => {
            let s = reserve(buf, pos, end, (l + 4) as usize)?;
            s[0] = 0xc8;
            s[1..3].copy_from_slice(&(l as u16).to_be_bytes());
            s[3] = typ_u;
            s[4..].copy_from_slice(v);
            Ok(())
        }
        _ => {
            let s = reserve(buf, pos, end, (l + 6) as usize)?;
            s[0] = 0xc9;
            s[1..5].copy_from_slice(&l.to_be_bytes());
            s[5] = typ_u;
            s[6..].copy_from_slice(v);
            Ok(())
        }
    }
}

pub fn encode_time(
    buf: &mut [u8],
    pos: &mut usize,
    end: usize,
    sec: i64,
    nsec: u32,
    be_compatible: bool,
) -> Result<()> {
    if be_compatible {
        return Err(Error::IllegalCall);
    }
    if nsec >= 1_000_000_000 {
        return Err(Error::ValueError);
    }
    if (sec as u64) & 0xfffffffc00000000 != 0 {
        let s = reserve(buf, pos, end, 15)?;
        s[0] = 0xc7;
        s[1] = 12;
        s[2] = 0xff;
        s[3..7].copy_from_slice(&nsec.to_be_bytes());
        s[7..15].copy_from_slice(&(sec as u64).to_be_bytes());
        return Ok(());
    }
    let data64 = ((nsec as u64) << 34) | (sec as u64);
    if data64 & 0xffffffff00000000 != 0 {
        let s = reserve(buf, pos, end, 10)?;
        s[0] = 0xd7;
        s[1] = 0xff;
        s[2..10].copy_from_slice(&data64.to_be_bytes());
        return Ok(());
    }
    let data32 = data64 as u32;
    let s = reserve(buf, pos, end, 6)?;
    s[0] = 0xd6;
    s[1] = 0xff;
    s[2..6].copy_from_slice(&data32.to_be_bytes());
    Ok(())
}

pub fn encode_insert(buf: &mut [u8], pos: &mut usize, end: usize, v: &[u8]) -> Result<()> {
    let s = reserve(buf, pos, end, v.len())?;
    s.copy_from_slice(v);
    Ok(())
}
