#!/usr/bin/env bash
# Build Rust port and run the UNMODIFIED CWPack module test against it.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
cd "$ROOT"
cargo build --release
clang -O2 -I "$ROOT/include" \
  -o "$CARGO_TARGET_DIR/release/cwpackModuleTest" \
  "$ROOT/tests/original/cwpack_module_test.c" \
  "$CARGO_TARGET_DIR/release/libcwpack.a" \
  -framework Security -framework CoreFoundation
"$CARGO_TARGET_DIR/release/cwpackModuleTest"
