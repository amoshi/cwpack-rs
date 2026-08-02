//! Read MessagePack file written by C (or Rust) and verify the canonical object.
//! Usage: cross_read <in.mp>

use cwpack::{cw_unpack_next, CwUnpackContext, ItemType};
use std::env;
use std::fs;
use std::process::ExitCode;

fn expect_str(uc: &mut CwUnpackContext<'_>, want: &str) -> Result<(), String> {
    cw_unpack_next(uc);
    if uc.return_code != 0 {
        return Err(format!("unpack str rc={}", uc.return_code));
    }
    if uc.item.type_code != ItemType::Str as i32 {
        return Err(format!("want str, got type {}", uc.item.type_code));
    }
    let got = uc
        .item_blob()
        .ok_or_else(|| "missing blob".to_string())?;
    if got != want.as_bytes() {
        return Err(format!(
            "str want {:?} got {:?}",
            want,
            String::from_utf8_lossy(got)
        ));
    }
    Ok(())
}

fn verify(buf: &[u8]) -> Result<(), String> {
    let mut uc = CwUnpackContext::new(buf);

    cw_unpack_next(&mut uc);
    if uc.return_code != 0 || uc.item.type_code != ItemType::Map as i32 || uc.item.size != 4 {
        return Err(format!(
            "map header rc={} type={} size={}",
            uc.return_code, uc.item.type_code, uc.item.size
        ));
    }

    expect_str(&mut uc, "compact")?;
    cw_unpack_next(&mut uc);
    if uc.return_code != 0
        || uc.item.type_code != ItemType::Boolean as i32
        || !uc.item.boolean
    {
        return Err("compact != true".into());
    }

    expect_str(&mut uc, "schema")?;
    cw_unpack_next(&mut uc);
    if uc.return_code != 0
        || uc.item.type_code != ItemType::PositiveInteger as i32
        || uc.item.u64 != 0
    {
        return Err("schema != 0".into());
    }

    expect_str(&mut uc, "name")?;
    expect_str(&mut uc, "demo")?;

    expect_str(&mut uc, "vals")?;
    cw_unpack_next(&mut uc);
    if uc.return_code != 0 || uc.item.type_code != ItemType::Array as i32 || uc.item.size != 3 {
        return Err("vals array header".into());
    }

    cw_unpack_next(&mut uc);
    if uc.return_code != 0
        || uc.item.type_code != ItemType::NegativeInteger as i32
        || uc.item.i64 != -32
    {
        return Err(format!("vals[0] want -32 got i64={}", uc.item.i64));
    }

    cw_unpack_next(&mut uc);
    if uc.return_code != 0
        || uc.item.type_code != ItemType::PositiveInteger as i32
        || uc.item.u64 != 255
    {
        return Err(format!("vals[1] want 255 got u64={}", uc.item.u64));
    }

    cw_unpack_next(&mut uc);
    if uc.return_code != 0 || uc.item.type_code != ItemType::Nil as i32 {
        return Err("vals[2] want nil".into());
    }

    // EOF
    cw_unpack_next(&mut uc);
    if uc.return_code != cwpack::Error::EndOfInput.code() {
        return Err(format!("expected END_OF_INPUT, rc={}", uc.return_code));
    }
    Ok(())
}

fn main() -> ExitCode {
    let path = env::args().nth(1).expect("usage: cross_read <in.mp>");
    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    match verify(&data) {
        Ok(()) => {
            eprintln!("rust verified OK ({path}, {} bytes)", data.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("rust verify FAIL: {e}");
            ExitCode::from(1)
        }
    }
}
