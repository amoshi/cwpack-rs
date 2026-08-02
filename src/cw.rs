//! C-shaped API: `CwPackContext` / `CwUnpackContext` + `cw_pack_*` / `cw_unpack_*`.
//!
//! Closest to copy-paste from CWPack C examples — only `&mut`, `b"..."` and types change.

use crate::error::Error;
use crate::item::ItemType;
use crate::pack;
use crate::unpack::{self, Decoded};

/// Like C `cw_pack_context` — owns a borrow of the output buffer.
pub struct CwPackContext<'a> {
    buf: &'a mut [u8],
    pos: usize,
    end: usize,
    pub be_compatible: bool,
    /// Sticky error code (`CWP_RC_*` / [`Error::code`]). `0` = ok.
    pub return_code: i32,
}

/// Like C `cw_unpack_context`.
pub struct CwUnpackContext<'a> {
    buf: &'a [u8],
    pos: usize,
    end: usize,
    pub return_code: i32,
    /// Last decoded item (updated by [`cw_unpack_next`]).
    pub item: Decoded,
}

impl<'a> CwPackContext<'a> {
    /// `cw_pack_context_init` without overflow handler (handler = null).
    pub fn new(data: &'a mut [u8]) -> Self {
        let end = data.len();
        Self {
            buf: data,
            pos: 0,
            end,
            be_compatible: false,
            return_code: Error::Ok.code(),
        }
    }

    /// Bytes written so far (`current - start` in C).
    pub fn len_packed(&self) -> usize {
        self.pos
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
}

impl<'a> CwUnpackContext<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let end = data.len();
        Self {
            buf: data,
            pos: 0,
            end,
            return_code: Error::Ok.code(),
            item: Decoded {
                type_code: ItemType::NotAnItem as i32,
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
            },
        }
    }

    /// Str/bin/ext payload of the last item, if any.
    pub fn item_blob(&self) -> Option<&[u8]> {
        let t = self.item.type_code;
        if t == ItemType::Str as i32
            || t == ItemType::Bin as i32
            || (-128..=127).contains(&t) && t != ItemType::Timestamp as i32
        {
            let off = self.item.blob_off;
            let len = self.item.blob_len as usize;
            self.buf.get(off..off + len)
        } else {
            None
        }
    }
}

fn pack_apply(pc: &mut CwPackContext<'_>, f: impl FnOnce(&mut [u8], &mut usize, usize) -> Result<(), Error>) {
    if pc.return_code != 0 {
        return;
    }
    match f(pc.buf, &mut pc.pos, pc.end) {
        Ok(()) => {}
        Err(e) => pc.return_code = e.code(),
    }
}

/// `int cw_pack_context_init(cw_pack_context*, void* data, unsigned long length, handler)`
///
/// Safe Rust: pass the mutable buffer; overflow handler is unused (`null`).
pub fn cw_pack_context_init<'a>(
    pack_context: &mut CwPackContext<'a>,
    data: &'a mut [u8],
) -> i32 {
    *pack_context = CwPackContext::new(data);
    pack_context.return_code
}

pub fn cw_pack_set_compatibility(pack_context: &mut CwPackContext<'_>, be_compatible: bool) {
    pack_context.be_compatible = be_compatible;
}

pub fn cw_pack_nil(pack_context: &mut CwPackContext<'_>) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_nil(buf, pos, end));
}

pub fn cw_pack_true(pack_context: &mut CwPackContext<'_>) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_bool(buf, pos, end, true));
}

pub fn cw_pack_false(pack_context: &mut CwPackContext<'_>) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_bool(buf, pos, end, false));
}

pub fn cw_pack_boolean(pack_context: &mut CwPackContext<'_>, b: bool) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_bool(buf, pos, end, b));
}

pub fn cw_pack_signed(pack_context: &mut CwPackContext<'_>, i: i64) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_signed(buf, pos, end, i));
}

pub fn cw_pack_unsigned(pack_context: &mut CwPackContext<'_>, i: u64) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_unsigned(buf, pos, end, i));
}

pub fn cw_pack_float(pack_context: &mut CwPackContext<'_>, f: f32) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_float(buf, pos, end, f));
}

pub fn cw_pack_double(pack_context: &mut CwPackContext<'_>, d: f64) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_double(buf, pos, end, d));
}

pub fn cw_pack_array_size(pack_context: &mut CwPackContext<'_>, n: u32) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_array_size(buf, pos, end, n));
}

pub fn cw_pack_map_size(pack_context: &mut CwPackContext<'_>, n: u32) {
    pack_apply(pack_context, |buf, pos, end| pack::encode_map_size(buf, pos, end, n));
}

/// `cw_pack_str(ctx, v, l)` — uses first `l` bytes of `v`.
pub fn cw_pack_str(pack_context: &mut CwPackContext<'_>, v: &[u8], l: u32) {
    let compat = pack_context.be_compatible;
    let n = (l as usize).min(v.len());
    let bytes = &v[..n];
    pack_apply(pack_context, |buf, pos, end| {
        pack::encode_str(buf, pos, end, bytes, compat)
    });
}

pub fn cw_pack_bin(pack_context: &mut CwPackContext<'_>, v: &[u8], l: u32) {
    let compat = pack_context.be_compatible;
    let n = (l as usize).min(v.len());
    let bytes = &v[..n];
    pack_apply(pack_context, |buf, pos, end| {
        pack::encode_bin(buf, pos, end, bytes, compat)
    });
}

pub fn cw_pack_ext(pack_context: &mut CwPackContext<'_>, type_: i8, v: &[u8], l: u32) {
    let compat = pack_context.be_compatible;
    let n = (l as usize).min(v.len());
    let bytes = &v[..n];
    pack_apply(pack_context, |buf, pos, end| {
        pack::encode_ext(buf, pos, end, type_, bytes, compat)
    });
}

pub fn cw_pack_time(pack_context: &mut CwPackContext<'_>, sec: i64, nsec: u32) {
    let compat = pack_context.be_compatible;
    pack_apply(pack_context, |buf, pos, end| {
        pack::encode_time(buf, pos, end, sec, nsec, compat)
    });
}

pub fn cw_pack_insert(pack_context: &mut CwPackContext<'_>, v: &[u8], l: u32) {
    let n = (l as usize).min(v.len());
    let bytes = &v[..n];
    pack_apply(pack_context, |buf, pos, end| pack::encode_insert(buf, pos, end, bytes));
}

// --- methods on context (same names, for `pc.cw_pack_*(...)`) ---

impl CwPackContext<'_> {
    pub fn cw_pack_set_compatibility(&mut self, be_compatible: bool) {
        cw_pack_set_compatibility(self, be_compatible);
    }
    pub fn cw_pack_nil(&mut self) {
        cw_pack_nil(self);
    }
    pub fn cw_pack_true(&mut self) {
        cw_pack_true(self);
    }
    pub fn cw_pack_false(&mut self) {
        cw_pack_false(self);
    }
    pub fn cw_pack_boolean(&mut self, b: bool) {
        cw_pack_boolean(self, b);
    }
    pub fn cw_pack_signed(&mut self, i: i64) {
        cw_pack_signed(self, i);
    }
    pub fn cw_pack_unsigned(&mut self, i: u64) {
        cw_pack_unsigned(self, i);
    }
    pub fn cw_pack_float(&mut self, f: f32) {
        cw_pack_float(self, f);
    }
    pub fn cw_pack_double(&mut self, d: f64) {
        cw_pack_double(self, d);
    }
    pub fn cw_pack_array_size(&mut self, n: u32) {
        cw_pack_array_size(self, n);
    }
    pub fn cw_pack_map_size(&mut self, n: u32) {
        cw_pack_map_size(self, n);
    }
    pub fn cw_pack_str(&mut self, v: &[u8], l: u32) {
        cw_pack_str(self, v, l);
    }
    pub fn cw_pack_bin(&mut self, v: &[u8], l: u32) {
        cw_pack_bin(self, v, l);
    }
    pub fn cw_pack_ext(&mut self, type_: i8, v: &[u8], l: u32) {
        cw_pack_ext(self, type_, v, l);
    }
    pub fn cw_pack_time(&mut self, sec: i64, nsec: u32) {
        cw_pack_time(self, sec, nsec);
    }
    pub fn cw_pack_insert(&mut self, v: &[u8], l: u32) {
        cw_pack_insert(self, v, l);
    }
}

pub fn cw_unpack_context_init<'a>(
    unpack_context: &mut CwUnpackContext<'a>,
    data: &'a [u8],
) -> i32 {
    *unpack_context = CwUnpackContext::new(data);
    unpack_context.return_code
}

pub fn cw_unpack_next(unpack_context: &mut CwUnpackContext<'_>) {
    if unpack_context.return_code != 0 {
        return;
    }
    match unpack::unpack_next(unpack_context.buf, &mut unpack_context.pos, unpack_context.end) {
        Ok(d) => unpack_context.item = d,
        Err(e) => {
            unpack_context.item.type_code = ItemType::NotAnItem as i32;
            unpack_context.return_code = e.code();
        }
    }
}

pub fn cw_skip_items(unpack_context: &mut CwUnpackContext<'_>, item_count: i64) {
    if unpack_context.return_code != 0 {
        return;
    }
    match unpack::skip_items(
        unpack_context.buf,
        &mut unpack_context.pos,
        unpack_context.end,
        item_count,
    ) {
        Ok(()) => {}
        Err(e) => {
            unpack_context.item.type_code = ItemType::NotAnItem as i32;
            unpack_context.return_code = e.code();
        }
    }
}

pub fn cw_look_ahead(unpack_context: &mut CwUnpackContext<'_>) -> i32 {
    if unpack_context.return_code != 0 {
        return ItemType::NotAnItem as i32;
    }
    match unpack::look_ahead(unpack_context.buf, unpack_context.pos, unpack_context.end) {
        Ok(t) => t,
        Err(e) => {
            unpack_context.item.type_code = ItemType::NotAnItem as i32;
            unpack_context.return_code = e.code();
            ItemType::NotAnItem as i32
        }
    }
}

impl CwUnpackContext<'_> {
    pub fn cw_unpack_next(&mut self) {
        cw_unpack_next(self);
    }
    pub fn cw_skip_items(&mut self, item_count: i64) {
        cw_skip_items(self, item_count);
    }
    pub fn cw_look_ahead(&mut self) -> i32 {
        cw_look_ahead(self)
    }
}
