#!/usr/bin/env bash
# Build C + Rust benches, run identical workload, write results.json
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CWPACK_SRC="${CWPACK_SRC:-$ROOT/../CWPack}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
RUNS="${RUNS:-20}"
WARMUP="${WARMUP:-2}"
# Must match C `-DITERATIONS=` and Rust `env ITERATIONS` (export for rust_bench).
export ITERATIONS="${ITERATIONS:-1000000}"

cd "$ROOT"
mkdir -p bench

echo "== build Rust =="
cargo build --release --example rust_bench

echo "== build C =="
if [[ ! -f "$CWPACK_SRC/src/cwpack.c" ]]; then
  echo "CWPack sources not found at $CWPACK_SRC (set CWPACK_SRC)" >&2
  exit 1
fi
clang -O3 -DITERATIONS="$ITERATIONS" \
  -I "$CWPACK_SRC/src" \
  -o "$CARGO_TARGET_DIR/release/c_bench" \
  bench/c_bench.c "$CWPACK_SRC/src/cwpack.c"

C_BIN="$CARGO_TARGET_DIR/release/c_bench"
R_BIN="$CARGO_TARGET_DIR/release/examples/rust_bench"

percentile() {
  # stdin: one float per line; args: p (0-100)
  local p="$1"
  sort -n | awk -v p="$p" '
    { a[NR]=$1 }
    END {
      if (NR==0) { print "nan"; exit }
      idx = int((p/100)*(NR-1))+1
      if (idx<1) idx=1
      if (idx>NR) idx=NR
      printf "%.6f", a[idx]
    }'
}

run_series() {
  local bin="$1" mode="$2" out="$3"
  : > "$out"
  local i
  for i in $(seq 1 "$WARMUP"); do
    "$bin" "$mode" >/dev/null
  done
  for i in $(seq 1 "$RUNS"); do
    "$bin" "$mode" | tee -a "$out" >/dev/null
  done
}

mean_of() {
  awk '{s+=$1;n++} END{if(n) printf "%.6f", s/n; else print "nan"}' "$1"
}

ops_per_run() {
  # pack loop: 3 ops * N; unpack loop: 3 ops * N
  python3 - <<PY
N=int("$ITERATIONS")
print(N*3 + N*3)
PY
}

echo "== timed series (C) =="
run_series "$C_BIN" timed /tmp/cwpack_c_timed.txt
echo "== timed series (Rust) =="
run_series "$R_BIN" timed /tmp/cwpack_r_timed.txt

echo "== startup series =="
run_series "$C_BIN" startup /tmp/cwpack_c_startup.txt
run_series "$R_BIN" startup /tmp/cwpack_r_startup.txt

echo "== RSS (one cold run each) =="
rss_kb() {
  local bin="$1"
  # macOS /usr/bin/time -l → maximum resident set size in bytes
  if /usr/bin/time -l "$bin" timed >/dev/null 2>/tmp/cwpack_time_err.txt; then
    :
  fi
  if rg -q "maximum resident set size" /tmp/cwpack_time_err.txt 2>/dev/null; then
    awk '/maximum resident set size/ {printf "%.0f", $1/1024; exit}' /tmp/cwpack_time_err.txt
  elif rg -q "Maximum resident set size" /tmp/cwpack_time_err.txt 2>/dev/null; then
    # GNU time -v sometimes in KB already
    awk -F: '/Maximum resident set size/ {gsub(/[^0-9]/,"",$2); print $2+0; exit}' /tmp/cwpack_time_err.txt
  else
    echo "null"
  fi
}

C_RSS="$(rss_kb "$C_BIN")"
R_RSS="$(rss_kb "$R_BIN")"

C_P50="$(percentile 50 </tmp/cwpack_c_timed.txt)"
C_P99="$(percentile 99 </tmp/cwpack_c_timed.txt)"
R_P50="$(percentile 50 </tmp/cwpack_r_timed.txt)"
R_P99="$(percentile 99 </tmp/cwpack_r_timed.txt)"
C_MEAN="$(mean_of /tmp/cwpack_c_timed.txt)"
R_MEAN="$(mean_of /tmp/cwpack_r_timed.txt)"
C_START="$(percentile 99 </tmp/cwpack_c_startup.txt)"
R_START="$(percentile 99 </tmp/cwpack_r_startup.txt)"
OPS="$(ops_per_run)"

C_TPUT="$(python3 -c "print(f'{$OPS / ($C_MEAN/1000):.0f}')")"
R_TPUT="$(python3 -c "print(f'{$OPS / ($R_MEAN/1000):.0f}')")"

MACHINE="$(uname -srm)"
CLANGV="$(clang --version | head -1)"
RUSTCV="$(rustc --version)"
CPU="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || (grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs) || echo unknown)"

python3 - <<PY
import json
from pathlib import Path
doc = {
  "workload": f"{int('$ITERATIONS')} pack(u16-range, str'bench', nil) buffer-reuse + {int('$ITERATIONS')} unpack of fixed 3-item msg",
  "ops_per_run": int("$OPS"),
  "iterations": int("$ITERATIONS"),
  "runs": int("$RUNS"),
  "warmup_runs": int("$WARMUP"),
  "original_c": {
    "p50_ms": float("$C_P50"),
    "p99_ms": float("$C_P99"),
    "mean_ms": float("$C_MEAN"),
    "rss_kb": None if "$C_RSS" == "null" else float("$C_RSS"),
    "throughput_ops_per_s": float("$C_TPUT"),
  },
  "cwpack_rs": {
    "p50_ms": float("$R_P50"),
    "p99_ms": float("$R_P99"),
    "mean_ms": float("$R_MEAN"),
    "rss_kb": None if "$R_RSS" == "null" else float("$R_RSS"),
    "throughput_ops_per_s": float("$R_TPUT"),
  },
  "startup_ms": {
    "original_c_p99": float("$C_START"),
    "cwpack_rs_p99": float("$R_START"),
    "note": "in-process init+nil (not process spawn)"
  },
  "machine": "$MACHINE",
  "cpu": "$CPU",
  "toolchains": {"clang": "$CLANGV", "rustc": "$RUSTCV"},
  "git_source_sha": "833fec93903f047ae5c47936f884ba27fc4c7a4c",
  "commands": {
    "run": "bench/run.sh",
    "c_build": "clang -O3 -I ../CWPack/src bench/c_bench.c ../CWPack/src/cwpack.c",
    "rust_build": "cargo build --release --example rust_bench"
  }
}
Path("bench/results.json").write_text(json.dumps(doc, indent=2) + "\n")
print(json.dumps(doc, indent=2))
PY

echo "Wrote bench/results.json"
