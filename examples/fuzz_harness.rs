//! Differential self-fuzz harness (60s by default).
//! `CWPACK_FUZZ_SECS=2 cargo run --release --example fuzz_harness`

use cwpack::pack;
use cwpack::unpack;
use std::env;
use std::time::{Duration, Instant};

fn roundtrip_unsigned(v: u64) -> bool {
    let mut buf = [0u8; 16];
    let end = buf.len();
    let mut pos = 0;
    if pack::encode_unsigned(&mut buf, &mut pos, end, v).is_err() {
        return false;
    }
    let written = pos;
    let mut pos = 0;
    match unpack::unpack_next(&buf, &mut pos, written) {
        Ok(d) => d.u64 == v || d.i64 as u64 == v,
        Err(_) => false,
    }
}

fn main() {
    let secs: u64 = env::var("CWPACK_FUZZ_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let start = Instant::now();
    let mut iters = 0u64;
    let mut divergences = 0u64;
    let limit = Duration::from_secs(secs);
    let mut seed = 0xC0FFEE_u64;
    while start.elapsed() < limit {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        if !roundtrip_unsigned(seed) {
            divergences += 1;
            eprintln!("diverge unsigned {seed}");
            break;
        }
        iters += 1;
    }
    println!(
        "differential_self_fuzz iters={iters} divergences={divergences} elapsed_ms={}",
        start.elapsed().as_millis()
    );
}
