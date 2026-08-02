#!/usr/bin/env bash
# Verify the Rust port (no C FFI).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
cd "$ROOT"
cargo test --release
echo "Note: upstream C module test is not linked (no c-abi)."
echo "Byte-level C vs Rust: ./extra-tests/run_json_diff.sh"
