# Benchmark methodology — cwpack-rs vs original CWPack

## Workload

- Pack/unpack 1_000_000 integers and short strings (same pattern as CWPack module/perf spirit).
- Measure wall time (p50/p99 over 20 runs), peak RSS via `/usr/bin/time -l` (macOS) or `/usr/bin/time -v` (Linux).
- Startup: time to `cw_pack_context_init` + one nil pack (micro-benchmark).

## Environment

Record: CPU model, OS, rustc version, clang version, `cargo build --release`.

## Honesty rules

- Report regressions; do not hide slower ports.
- Compare against CWPack C built with `-O3` from the pinned SHA.
- Throughput alone is insufficient — include p99 and RSS.

## Results

See `results.json` for the latest recorded numbers (fill after a timed run on the submission machine).
