//! C ABI shim — the only module that uses `unsafe`.
//! Core encode/decode logic lives in safe `pack` / `unpack`.

use std::os::raw::{c_char, c_int, c_ulong, c_void};
use std::slice;

use crate::error::Error;
use crate::item::ItemType;
use crate::pack;
use crate::unpack;

pub type PackOverflowHandler = Option<extern "C" fn(*mut PackContext, c_ulong) -> c_int>;
pub type PackFlushHandler = Option<extern "C" fn(*mut PackContext) -> c_int>;
pub type UnpackUnderflowHandler = Option<extern "C" fn(*mut UnpackContext, c_ulong) -> c_int>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Blob {
    pub start: *const c_void,
    pub length: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Container {
    pub size: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ItemAs {
    pub boolean: bool,
    pub u64: u64,
    pub i64: i64,
    pub real: f32,
    pub long_real: f64,
    pub array: Container,
    pub map: Container,
    pub str: Blob,
    pub bin: Blob,
    pub ext: Blob,
    pub time: Timespec,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Item {
    pub type_: c_int,
    pub as_: ItemAs,
}

#[repr(C)]
pub struct PackContext {
    pub current: *mut u8,
    pub start: *mut u8,
    pub end: *mut u8,
    pub be_compatible: bool,
    pub return_code: c_int,
    pub err_no: c_int,
    pub handle_pack_overflow: PackOverflowHandler,
    pub handle_flush: PackFlushHandler,
}

#[repr(C)]
pub struct UnpackContext {
    pub item: Item,
    pub start: *mut u8,
    pub current: *mut u8,
    pub end: *mut u8,
    pub return_code: c_int,
    pub err_no: c_int,
    pub handle_unpack_underflow: UnpackUnderflowHandler,
}

fn set_err(rc: &mut c_int, e: Error) {
    *rc = e.code();
}

unsafe fn pack_region(ctx: &mut PackContext) -> Result<(&mut [u8], usize, usize), Error> {
    if ctx.start.is_null() {
        return Err(Error::BufferOverflow);
    }
    let cap = ctx.end.offset_from(ctx.start) as usize;
    let pos = ctx.current.offset_from(ctx.start) as usize;
    let buf = slice::from_raw_parts_mut(ctx.start, cap);
    Ok((buf, pos, cap))
}

unsafe fn pack_commit(ctx: &mut PackContext, pos: usize) {
    ctx.current = ctx.start.add(pos);
}

unsafe fn pack_fn(ctx: *mut PackContext, f: impl Fn(&mut [u8], &mut usize, usize) -> Result<(), Error>) {
    let c = &mut *ctx;
    if c.return_code != 0 {
        return;
    }
    for round in 0..3 {
        let c = &mut *ctx;
        let (buf, mut pos, end) = match pack_region(c) {
            Ok(v) => v,
            Err(e) => {
                set_err(&mut c.return_code, e);
                return;
            }
        };
        match f(buf, &mut pos, end) {
            Ok(()) => {
                pack_commit(c, pos);
                return;
            }
            Err(Error::BufferOverflow) if round < 2 => {
                match c.handle_pack_overflow {
                    Some(h) => {
                        let rc = h(ctx, 256);
                        if rc != 0 {
                            set_err(&mut (*ctx).return_code, Error::from_code(rc));
                            return;
                        }
                    }
                    None => {
                        set_err(&mut c.return_code, Error::BufferOverflow);
                        return;
                    }
                }
            }
            Err(e) => {
                set_err(&mut c.return_code, e);
                return;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_context_init(
    pack_context: *mut PackContext,
    data: *mut c_void,
    length: c_ulong,
    hpo: PackOverflowHandler,
) -> c_int {
    let ctx = &mut *pack_context;
    ctx.start = data as *mut u8;
    ctx.current = data as *mut u8;
    ctx.end = (data as *mut u8).add(length as usize);
    ctx.be_compatible = false;
    ctx.err_no = 0;
    ctx.handle_pack_overflow = hpo;
    ctx.handle_flush = None;
    ctx.return_code = Error::Ok.code();
    ctx.return_code
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_set_compatibility(pack_context: *mut PackContext, be_compatible: bool) {
    (*pack_context).be_compatible = be_compatible;
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_set_flush_handler(
    pack_context: *mut PackContext,
    handle_flush: PackFlushHandler,
) {
    (*pack_context).handle_flush = handle_flush;
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_flush(pack_context: *mut PackContext) {
    let ctx = &mut *pack_context;
    if ctx.return_code == 0 {
        ctx.return_code = match ctx.handle_flush {
            Some(h) => h(pack_context),
            None => Error::IllegalCall.code(),
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_unsigned(pack_context: *mut PackContext, i: u64) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_unsigned(buf, pos, end, i));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_signed(pack_context: *mut PackContext, i: i64) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_signed(buf, pos, end, i));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_float(pack_context: *mut PackContext, f: f32) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_float(buf, pos, end, f));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_double(pack_context: *mut PackContext, d: f64) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_double(buf, pos, end, d));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_nil(pack_context: *mut PackContext) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_nil(buf, pos, end));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_true(pack_context: *mut PackContext) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_bool(buf, pos, end, true));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_false(pack_context: *mut PackContext) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_bool(buf, pos, end, false));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_boolean(pack_context: *mut PackContext, b: bool) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_bool(buf, pos, end, b));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_array_size(pack_context: *mut PackContext, n: u32) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_array_size(buf, pos, end, n));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_map_size(pack_context: *mut PackContext, n: u32) {
    pack_fn(pack_context, |buf, pos, end| pack::encode_map_size(buf, pos, end, n));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_str(pack_context: *mut PackContext, v: *const c_char, l: u32) {
    let compat = (*pack_context).be_compatible;
    let bytes = if l == 0 {
        &[][..]
    } else {
        slice::from_raw_parts(v as *const u8, l as usize)
    };
    pack_fn(pack_context, |buf, pos, end| pack::encode_str(buf, pos, end, bytes, compat));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_bin(pack_context: *mut PackContext, v: *const c_void, l: u32) {
    let compat = (*pack_context).be_compatible;
    let bytes = if l == 0 {
        &[][..]
    } else {
        slice::from_raw_parts(v as *const u8, l as usize)
    };
    pack_fn(pack_context, |buf, pos, end| pack::encode_bin(buf, pos, end, bytes, compat));
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_ext(
    pack_context: *mut PackContext,
    type_: i8,
    v: *const c_void,
    l: u32,
) {
    let compat = (*pack_context).be_compatible;
    let bytes = if l == 0 {
        &[][..]
    } else {
        slice::from_raw_parts(v as *const u8, l as usize)
    };
    pack_fn(pack_context, |buf, pos, end| {
        pack::encode_ext(buf, pos, end, type_, bytes, compat)
    });
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_time(pack_context: *mut PackContext, sec: i64, nsec: u32) {
    let compat = (*pack_context).be_compatible;
    pack_fn(pack_context, |buf, pos, end| {
        pack::encode_time(buf, pos, end, sec, nsec, compat)
    });
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_insert(pack_context: *mut PackContext, v: *const c_void, l: u32) {
    let bytes = if l == 0 {
        &[][..]
    } else {
        slice::from_raw_parts(v as *const u8, l as usize)
    };
    // insert does not check return_code first in C — still packs via reserve
    pack_fn(pack_context, |buf, pos, end| pack::encode_insert(buf, pos, end, bytes));
}

unsafe fn unpack_region(ctx: &UnpackContext) -> Result<(&[u8], usize, usize), Error> {
    if ctx.start.is_null() {
        return Err(Error::EndOfInput);
    }
    let cap = ctx.end.offset_from(ctx.start) as usize;
    let pos = ctx.current.offset_from(ctx.start) as usize;
    let buf = slice::from_raw_parts(ctx.start, cap);
    Ok((buf, pos, cap))
}

unsafe fn apply_decoded(ctx: &mut UnpackContext, d: &unpack::Decoded) {
    ctx.item.type_ = d.type_code;
    let base = ctx.start;
    match d.type_code {
        x if x == ItemType::Nil as i32 => {}
        x if x == ItemType::Boolean as i32 => ctx.item.as_.boolean = d.boolean,
        x if x == ItemType::PositiveInteger as i32 => {
            ctx.item.as_.u64 = d.u64;
        }
        x if x == ItemType::NegativeInteger as i32 => {
            ctx.item.as_.i64 = d.i64;
        }
        x if x == ItemType::Float as i32 => ctx.item.as_.real = d.real,
        x if x == ItemType::Double as i32 => {
            ctx.item.as_.u64 = d.u64;
            ctx.item.as_.long_real = d.long_real;
        }
        x if x == ItemType::Array as i32 => ctx.item.as_.array = Container { size: d.size },
        x if x == ItemType::Map as i32 => ctx.item.as_.map = Container { size: d.size },
        x if x == ItemType::Str as i32 => {
            ctx.item.as_.str = Blob {
                start: base.add(d.blob_off) as *const c_void,
                length: d.blob_len,
            };
        }
        x if x == ItemType::Bin as i32 => {
            ctx.item.as_.bin = Blob {
                start: base.add(d.blob_off) as *const c_void,
                length: d.blob_len,
            };
        }
        x if x == ItemType::Timestamp as i32 => {
            ctx.item.as_.time = Timespec {
                tv_sec: d.time_sec,
                tv_nsec: d.time_nsec,
            };
        }
        _ => {
            // EXT tag stored as type_
            ctx.item.as_.ext = Blob {
                start: base.add(d.blob_off) as *const c_void,
                length: d.blob_len,
            };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_context_init(
    unpack_context: *mut UnpackContext,
    data: *const c_void,
    length: c_ulong,
    huu: UnpackUnderflowHandler,
) -> c_int {
    let ctx = &mut *unpack_context;
    ctx.start = data as *mut u8;
    ctx.current = data as *mut u8;
    ctx.end = (data as *mut u8).add(length as usize);
    ctx.return_code = Error::Ok.code();
    ctx.err_no = 0;
    ctx.handle_unpack_underflow = huu;
    ctx.item.type_ = ItemType::NotAnItem as c_int;
    ctx.return_code
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next(unpack_context: *mut UnpackContext) {
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return;
    }
    // Underflow handler support: try decode; on EndOfInput/Underflow call handler & retry.
    for round in 0..3 {
        let (buf, mut pos, end) = match unpack_region(ctx) {
            Ok(v) => v,
            Err(e) => {
                ctx.item.type_ = ItemType::NotAnItem as c_int;
                set_err(&mut ctx.return_code, e);
                return;
            }
        };
        match unpack::unpack_next(buf, &mut pos, end) {
            Ok(d) => {
                apply_decoded(ctx, &d);
                ctx.current = ctx.start.add(pos);
                return;
            }
            Err(e @ (Error::EndOfInput | Error::BufferUnderflow)) if round < 2 => {
                match ctx.handle_unpack_underflow {
                    Some(h) => {
                        let rc = h(unpack_context, 64);
                        if rc != 0 {
                            ctx.item.type_ = ItemType::NotAnItem as c_int;
                            if rc == Error::EndOfInput.code() {
                                set_err(&mut ctx.return_code, e);
                            } else {
                                set_err(&mut ctx.return_code, Error::from_code(rc));
                            }
                            return;
                        }
                    }
                    None => {
                        ctx.item.type_ = ItemType::NotAnItem as c_int;
                        set_err(&mut ctx.return_code, e);
                        return;
                    }
                }
            }
            Err(e) => {
                ctx.item.type_ = ItemType::NotAnItem as c_int;
                set_err(&mut ctx.return_code, e);
                return;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_skip_items(unpack_context: *mut UnpackContext, item_count: i64) {
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return;
    }
    let (buf, mut pos, end) = match unpack_region(ctx) {
        Ok(v) => v,
        Err(e) => {
            ctx.item.type_ = ItemType::NotAnItem as c_int;
            set_err(&mut ctx.return_code, e);
            return;
        }
    };
    match unpack::skip_items(buf, &mut pos, end, item_count) {
        Ok(()) => ctx.current = ctx.start.add(pos),
        Err(e) => {
            ctx.item.type_ = ItemType::NotAnItem as c_int;
            set_err(&mut ctx.return_code, e);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_look_ahead(unpack_context: *mut UnpackContext) -> c_int {
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return ItemType::NotAnItem as c_int;
    }
    let (buf, pos, end) = match unpack_region(ctx) {
        Ok(v) => v,
        Err(e) => {
            ctx.item.type_ = ItemType::NotAnItem as c_int;
            set_err(&mut ctx.return_code, e);
            return ItemType::NotAnItem as c_int;
        }
    };
    match unpack::look_ahead(buf, pos, end) {
        Ok(t) => t,
        Err(e) => {
            ctx.item.type_ = ItemType::NotAnItem as c_int;
            set_err(&mut ctx.return_code, e);
            ItemType::NotAnItem as c_int
        }
    }
}
