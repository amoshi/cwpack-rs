//! CWPack utils (goodies) — typed unpack helpers + opt packers.

use std::os::raw::c_int;

use crate::error::Error;
use crate::ffi::{cw_pack_double, cw_pack_float, cw_pack_signed, cw_pack_time, cw_unpack_next};
use crate::ffi::{PackContext, UnpackContext};
use crate::item::ItemType;

const NIL: i32 = ItemType::Nil as i32;
const BOOLEAN: i32 = ItemType::Boolean as i32;
const POS: i32 = ItemType::PositiveInteger as i32;
const NEG: i32 = ItemType::NegativeInteger as i32;
const FLOAT: i32 = ItemType::Float as i32;
const DOUBLE: i32 = ItemType::Double as i32;
const STR: i32 = ItemType::Str as i32;
const BIN: i32 = ItemType::Bin as i32;
const ARRAY: i32 = ItemType::Array as i32;
const MAP: i32 = ItemType::Map as i32;
const TS: i32 = ItemType::Timestamp as i32;

#[no_mangle]
pub unsafe extern "C" fn cw_pack_double_opt(pack_context: *mut PackContext, d: f64) {
    let i = d as i32;
    if (i as f64) == d && (i as i64) >= i32::MIN as i64 && (i as i64) <= u32::MAX as i64 {
        cw_pack_signed(pack_context, i as i64);
        return;
    }
    let f = d as f32;
    if (f as f64) == d {
        cw_pack_float(pack_context, f);
    } else {
        cw_pack_double(pack_context, d);
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_float_opt(pack_context: *mut PackContext, f: f32) {
    let i = f as i32;
    if (i as f32) == f && i >= i16::MIN as i32 && i <= u16::MAX as i32 {
        cw_pack_signed(pack_context, i as i64);
    } else {
        cw_pack_float(pack_context, f);
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_pack_time_interval(pack_context: *mut PackContext, ti: f64) {
    let sec = ti.floor() as i64;
    let nsec = ((ti - sec as f64) * 1_000_000_000.0) as u32;
    cw_pack_time(pack_context, sec, nsec);
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_float(unpack_context: *mut UnpackContext) -> f32 {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0.0;
    }
    match ctx.item.type_ {
        POS => ctx.item.as_.u64 as f32,
        NEG => ctx.item.as_.i64 as f32,
        FLOAT => ctx.item.as_.real,
        DOUBLE => ctx.item.as_.long_real as f32,
        _ => {
            ctx.return_code = Error::TypeError.code();
            0.0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_double(unpack_context: *mut UnpackContext) -> f64 {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0.0;
    }
    match ctx.item.type_ {
        POS => ctx.item.as_.u64 as f64,
        NEG => ctx.item.as_.i64 as f64,
        FLOAT => ctx.item.as_.real as f64,
        DOUBLE => ctx.item.as_.long_real,
        _ => {
            ctx.return_code = Error::TypeError.code();
            0.0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_nil(unpack_context: *mut UnpackContext) {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return;
    }
    if ctx.item.type_ != NIL {
        ctx.return_code = Error::TypeError.code();
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_boolean(unpack_context: *mut UnpackContext) -> bool {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return false;
    }
    if ctx.item.type_ == BOOLEAN {
        ctx.item.as_.boolean
    } else {
        ctx.return_code = Error::TypeError.code();
        false
    }
}

macro_rules! unpack_signed {
    ($name:ident, $ty:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(unpack_context: *mut UnpackContext) -> $ty {
            cw_unpack_next(unpack_context);
            let ctx = &mut *unpack_context;
            if ctx.return_code != 0 {
                return 0 as $ty;
            }
            match ctx.item.type_ {
                POS => {
                    if ctx.item.as_.u64 <= (<$ty>::MAX as u64) {
                        ctx.item.as_.i64 as $ty
                    } else {
                        ctx.return_code = Error::ValueError.code();
                        0 as $ty
                    }
                }
                NEG => {
                    if ctx.item.as_.i64 >= (<$ty>::MIN as i64) {
                        ctx.item.as_.i64 as $ty
                    } else {
                        ctx.return_code = Error::ValueError.code();
                        0 as $ty
                    }
                }
                _ => {
                    ctx.return_code = Error::TypeError.code();
                    0 as $ty
                }
            }
        }
    };
}

unpack_signed!(cw_unpack_next_signed64, i64);
unpack_signed!(cw_unpack_next_signed32, i32);
unpack_signed!(cw_unpack_next_signed16, i16);
unpack_signed!(cw_unpack_next_signed8, i8);

macro_rules! unpack_unsigned {
    ($name:ident, $ty:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(unpack_context: *mut UnpackContext) -> $ty {
            cw_unpack_next(unpack_context);
            let ctx = &mut *unpack_context;
            if ctx.return_code != 0 {
                return 0 as $ty;
            }
            if ctx.item.type_ == POS {
                if ctx.item.as_.u64 <= (<$ty>::MAX as u64) {
                    ctx.item.as_.u64 as $ty
                } else {
                    ctx.return_code = Error::ValueError.code();
                    0 as $ty
                }
            } else {
                ctx.return_code = Error::TypeError.code();
                0 as $ty
            }
        }
    };
}

unpack_unsigned!(cw_unpack_next_unsigned64, u64);
unpack_unsigned!(cw_unpack_next_unsigned32, u32);
unpack_unsigned!(cw_unpack_next_unsigned16, u16);
unpack_unsigned!(cw_unpack_next_unsigned8, u8);

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_time_interval(unpack_context: *mut UnpackContext) -> f64 {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0.0;
    }
    if ctx.item.type_ == TS {
        ctx.item.as_.time.tv_sec as f64 + (ctx.item.as_.time.tv_nsec as f64) / 1_000_000_000.0
    } else {
        ctx.return_code = Error::TypeError.code();
        0.0
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_str_lengh(unpack_context: *mut UnpackContext) -> c_int {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0;
    }
    if ctx.item.type_ == STR {
        ctx.item.as_.str.length as c_int
    } else {
        ctx.return_code = Error::TypeError.code();
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_bin_lengh(unpack_context: *mut UnpackContext) -> c_int {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0;
    }
    if ctx.item.type_ == BIN {
        ctx.item.as_.bin.length as c_int
    } else {
        ctx.return_code = Error::TypeError.code();
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_array_size(unpack_context: *mut UnpackContext) -> c_int {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0;
    }
    if ctx.item.type_ == ARRAY {
        ctx.item.as_.array.size as c_int
    } else {
        ctx.return_code = Error::TypeError.code();
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn cw_unpack_next_map_size(unpack_context: *mut UnpackContext) -> c_int {
    cw_unpack_next(unpack_context);
    let ctx = &mut *unpack_context;
    if ctx.return_code != 0 {
        return 0;
    }
    if ctx.item.type_ == MAP {
        ctx.item.as_.map.size as c_int
    } else {
        ctx.return_code = Error::TypeError.code();
        0
    }
}
