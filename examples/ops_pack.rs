//! Pack an ops stream (from extra-tests/json_to_ops.py) with cwpack Rust API.
//! Usage: ops_pack < ops.txt > out.mp

use cwpack::pack;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    let mut buf = vec![0u8; 64 * 1024 * 1024];
    let end = buf.len();
    let mut pos = 0usize;

    let mut i = 0;
    while i < input.len() {
        let line_end = input[i..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| i + p)
            .unwrap_or(input.len());
        let line = std::str::from_utf8(&input[i..line_end]).expect("utf8 op line");
        i = if line_end < input.len() {
            line_end + 1
        } else {
            input.len()
        };
        if line.is_empty() {
            continue;
        }
        if line == "NIL" {
            pack::encode_nil(&mut buf, &mut pos, end).unwrap();
        } else if let Some(rest) = line.strip_prefix("BOOL ") {
            pack::encode_bool(&mut buf, &mut pos, end, rest != "0").unwrap();
        } else if let Some(rest) = line.strip_prefix("U64 ") {
            pack::encode_unsigned(&mut buf, &mut pos, end, rest.parse().unwrap()).unwrap();
        } else if let Some(rest) = line.strip_prefix("I64 ") {
            pack::encode_signed(&mut buf, &mut pos, end, rest.parse().unwrap()).unwrap();
        } else if let Some(rest) = line.strip_prefix("F64BITS ") {
            let bits: u64 = rest.parse().unwrap();
            pack::encode_double(&mut buf, &mut pos, end, f64::from_bits(bits)).unwrap();
        } else if let Some(rest) = line.strip_prefix("STR ") {
            let len: usize = rest.parse().unwrap();
            if i + len > input.len() {
                panic!("short str");
            }
            let s = &input[i..i + len];
            i += len;
            if i < input.len() && input[i] == b'\n' {
                i += 1;
            }
            pack::encode_str(&mut buf, &mut pos, end, s, false).unwrap();
        } else if let Some(rest) = line.strip_prefix("ARR ") {
            pack::encode_array_size(&mut buf, &mut pos, end, rest.parse().unwrap()).unwrap();
        } else if let Some(rest) = line.strip_prefix("MAP ") {
            pack::encode_map_size(&mut buf, &mut pos, end, rest.parse().unwrap()).unwrap();
        } else {
            panic!("unknown op: {line}");
        }
    }

    io::stdout().write_all(&buf[..pos])?;
    Ok(())
}
