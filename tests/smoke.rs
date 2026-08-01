//! Port-side smoke tests (safe API).

use cwpack::pack;
use cwpack::unpack;

#[test]
fn pack_unpack_nil() {
    let mut buf = [0u8; 8];
    let end = buf.len();
    let mut pos = 0;
    pack::encode_nil(&mut buf, &mut pos, end).unwrap();
    assert_eq!(pos, 1);
    assert_eq!(buf[0], 0xc0);
    let mut p = 0;
    let d = unpack::unpack_next(&buf, &mut p, pos).unwrap();
    assert_eq!(d.type_code, cwpack::ItemType::Nil as i32);
}

#[test]
fn pack_map_example() {
    let mut buf = [0u8; 32];
    let end = buf.len();
    let mut pos = 0;
    pack::encode_map_size(&mut buf, &mut pos, end, 2).unwrap();
    pack::encode_str(&mut buf, &mut pos, end, b"compact", false).unwrap();
    pack::encode_bool(&mut buf, &mut pos, end, true).unwrap();
    pack::encode_str(&mut buf, &mut pos, end, b"schema", false).unwrap();
    pack::encode_unsigned(&mut buf, &mut pos, end, 0).unwrap();
    assert_eq!(pos, 18);
}
