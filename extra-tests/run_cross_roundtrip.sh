#!/usr/bin/env bash
# Cross-language MessagePack roundtrip:
#   1) Rust packs object → file
#   2) C reads file and verifies fields
#   3) C packs same object → file
#   4) Rust reads file and verifies fields
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CWPACK_SRC="${CWPACK_SRC:-$ROOT/../CWPack}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
OUT="$ROOT/extra-tests/out"
mkdir -p "$OUT"

cd "$ROOT"
cargo build --release --example cross_write --example cross_read

clang -O3 -I "$CWPACK_SRC/src" \
  -o "$CARGO_TARGET_DIR/release/cross_write_c" \
  extra-tests/cross_write.c "$CWPACK_SRC/src/cwpack.c"
clang -O3 -I "$CWPACK_SRC/src" \
  -o "$CARGO_TARGET_DIR/release/cross_read_c" \
  extra-tests/cross_read.c "$CWPACK_SRC/src/cwpack.c"

RS_WRITE="$CARGO_TARGET_DIR/release/examples/cross_write"
RS_READ="$CARGO_TARGET_DIR/release/examples/cross_read"
C_WRITE="$CARGO_TARGET_DIR/release/cross_write_c"
C_READ="$CARGO_TARGET_DIR/release/cross_read_c"

FROM_RS="$OUT/cross_from_rust.mp"
FROM_C="$OUT/cross_from_c.mp"

echo "== 1) Rust write → 2) C verify"
"$RS_WRITE" "$FROM_RS"
"$C_READ" "$FROM_RS"

echo "== 3) C write → 4) Rust verify"
"$C_WRITE" "$FROM_C"
"$RS_READ" "$FROM_C"

# Optional: files from both writers should be byte-identical (same object + order)
if cmp -s "$FROM_RS" "$FROM_C"; then
  echo "OK  Rust and C writers produced identical bytes ($(wc -c < "$FROM_RS" | tr -d ' '))"
else
  echo "WARN writers differ (verify still passed); dumping sizes"
  ls -la "$FROM_RS" "$FROM_C"
  exit 1
fi

echo "cross-language roundtrip: all OK"
