//! Same workload as `bench/c_bench.c` (safe Rust API).
//! Run: cargo run --release --example rust_bench -- timed|startup
//! Iterations: env `ITERATIONS` (default 1_000_000), same as C `-DITERATIONS=`.

use cwpack::pack;
use cwpack::unpack;
use std::env;
use std::time::Instant;

fn iterations() -> i32 {
    env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000)
}

fn workload(buf: &mut [u8], n: i32) {
    let end = buf.len();
    let s = b"bench";

    for i in 0..n {
        let mut pos = 0;
        pack::encode_unsigned(buf, &mut pos, end, (i as u64) & 0xffff).unwrap();
        pack::encode_str(buf, &mut pos, end, s, false).unwrap();
        pack::encode_nil(buf, &mut pos, end).unwrap();
    }

    let mut pos = 0;
    pack::encode_unsigned(buf, &mut pos, end, 42).unwrap();
    pack::encode_str(buf, &mut pos, end, s, false).unwrap();
    pack::encode_nil(buf, &mut pos, end).unwrap();
    let packed = pos;

    for _ in 0..n {
        let mut p = 0;
        unpack::unpack_next(buf, &mut p, packed).unwrap();
        unpack::unpack_next(buf, &mut p, packed).unwrap();
        unpack::unpack_next(buf, &mut p, packed).unwrap();
    }
}

fn main() {
    let mode = env::args().nth(1).unwrap_or_else(|| "timed".into());
    let mut buf = [0u8; 65536];
    let n = iterations();

    if mode == "startup" {
        let t0 = Instant::now();
        let end = buf.len();
        let mut pos = 0;
        pack::encode_nil(&mut buf, &mut pos, end).unwrap();
        println!("{:.6}", t0.elapsed().as_secs_f64() * 1e3);
        return;
    }

    let t0 = Instant::now();
    workload(&mut buf, n);
    println!("{:.6}", t0.elapsed().as_secs_f64() * 1e3);
}
