//! Port-side smoke tests (C-like API).

use cwpack::{
    cw_pack_boolean, cw_pack_map_size, cw_pack_nil, cw_pack_str, cw_pack_unsigned,
    cw_unpack_next, CwPackContext, CwUnpackContext, ItemType,
};

#[test]
fn pack_unpack_nil() {
    let mut buf = [0u8; 8];
    let n = {
        let mut pc = CwPackContext::new(&mut buf);
        cw_pack_nil(&mut pc);
        assert_eq!(pc.return_code, 0);
        assert_eq!(pc.len_packed(), 1);
        pc.len_packed()
    };
    assert_eq!(buf[0], 0xc0);

    let mut uc = CwUnpackContext::new(&buf[..n]);
    cw_unpack_next(&mut uc);
    assert_eq!(uc.return_code, 0);
    assert_eq!(uc.item.type_code, ItemType::Nil as i32);
}

#[test]
fn pack_map_example_freefns() {
    let mut buffer = [0u8; 32];
    let mut pc = CwPackContext::new(&mut buffer);

    cw_pack_map_size(&mut pc, 2);
    cw_pack_str(&mut pc, b"compact", 7);
    cw_pack_boolean(&mut pc, true);
    cw_pack_str(&mut pc, b"schema", 6);
    cw_pack_unsigned(&mut pc, 0);

    assert_eq!(pc.return_code, 0);
    assert_eq!(pc.len_packed(), 18);
}

#[test]
fn pack_map_example_methods() {
    let mut buffer = [0u8; 32];
    let mut pc = CwPackContext::new(&mut buffer);

    pc.cw_pack_map_size(2);
    pc.cw_pack_str(b"compact", 7);
    pc.cw_pack_boolean(true);
    pc.cw_pack_str(b"schema", 6);
    pc.cw_pack_unsigned(0);

    assert_eq!(pc.return_code, 0);
    assert_eq!(pc.len_packed(), 18);
}
