# Benchmark methodology — cwpack-rs vs original CWPack

## Workload (identical C and Rust)

Per timed run (`ITERATIONS=1_000_000`, default):

1. **Pack loop:** 1e6 × (`unsigned` in 0..65535, `str "bench"`, `nil`), reusing a 64KiB buffer (cursor reset each trio).
2. **Unpack loop:** pack one fixed 3-item message, then unpack it 1e6 times (3 `unpack_next` each).

Ops/run = `2 * 3 * ITERATIONS` = 6e6.

Sources:

- C: `bench/c_bench.c` linked with `../CWPack/src/cwpack.c` (`-O3`)
- Rust: `examples/rust_bench.rs` safe API (`cargo build --release`)

## Procedure

```bash
# from cwpack-rs root; CWPack checkout next to it (or CWPACK_SRC=...)
chmod +x bench/run.sh
./bench/run.sh
```

Defaults: `WARMUP=2`, `RUNS=20`. Override with env vars.

Script writes `bench/results.json` with:

- **p50_ms / p99_ms / mean_ms** wall time (monotonic clock)
- **throughput_ops_per_s** = ops_per_run / mean_seconds
- **rss_kb** from `/usr/bin/time -l` (macOS) maximum RSS
- **startup_ms** p99 of in-process `init`/first nil (not process spawn)

## Honesty

- Same machine, consecutive C then Rust series.
- Regressions reported as-is.
- Source pin: `833fec93903f047ae5c47936f884ba27fc4c7a4c`
