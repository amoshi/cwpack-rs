//! Write a fixed MessagePack "object" to a file (Rust → C reader).
//! Usage: cross_write <out.mp>

use cwpack::{
    cw_pack_array_size, cw_pack_boolean, cw_pack_map_size, cw_pack_nil, cw_pack_signed,
    cw_pack_str, cw_pack_unsigned, CwPackContext,
};
use std::env;
use std::fs;
use std::process::ExitCode;

/// Canonical object (same layout as extra-tests/cross_write.c):
/// map(4):
///   "compact" -> true
///   "schema"  -> 0
///   "name"    -> "demo"
///   "vals"    -> array(3): -32, 255, nil
fn pack_object(buf: &mut [u8]) -> Result<usize, i32> {
    let mut pc = CwPackContext::new(buf);
    cw_pack_map_size(&mut pc, 4);

    cw_pack_str(&mut pc, b"compact", 7);
    cw_pack_boolean(&mut pc, true);

    cw_pack_str(&mut pc, b"schema", 6);
    cw_pack_unsigned(&mut pc, 0);

    cw_pack_str(&mut pc, b"name", 4);
    cw_pack_str(&mut pc, b"demo", 4);

    cw_pack_str(&mut pc, b"vals", 4);
    cw_pack_array_size(&mut pc, 3);
    cw_pack_signed(&mut pc, -32);
    cw_pack_unsigned(&mut pc, 255);
    cw_pack_nil(&mut pc);

    if pc.return_code != 0 {
        return Err(pc.return_code);
    }
    Ok(pc.len_packed())
}

fn main() -> ExitCode {
    let path = env::args().nth(1).expect("usage: cross_write <out.mp>");
    let mut buf = [0u8; 256];
    let n = match pack_object(&mut buf) {
        Ok(n) => n,
        Err(rc) => {
            eprintln!("pack failed rc={rc}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = fs::write(&path, &buf[..n]) {
        eprintln!("write {path}: {e}");
        return ExitCode::from(3);
    }
    eprintln!("rust wrote {n} bytes -> {path}");
    ExitCode::SUCCESS
}
