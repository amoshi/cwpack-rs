#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
cd "$ROOT"
CWPACK_FUZZ_SECS="${CWPACK_FUZZ_SECS:-60}" cargo run --release --example fuzz_harness | tee "$ROOT/fuzz/log.txt"
