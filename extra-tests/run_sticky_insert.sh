#!/usr/bin/env bash
# Differential demo of CWPack sticky-error bug in cw_pack_insert:
#   - stock C CWPack: insert WRITES after sticky ILLEGAL_CALL  → exit 1 (bug)
#   - cwpack-rs:      insert is a no-op after sticky error     → exit 0 (fixed)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CWPACK_SRC="${CWPACK_SRC:-$ROOT/../CWPack}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

cd "$ROOT"
cargo build --release --example sticky_insert_ok

clang -O0 -I "$CWPACK_SRC/src" \
  -o "$CARGO_TARGET_DIR/release/sticky_insert_bug_c" \
  extra-tests/sticky_insert_bug.c "$CWPACK_SRC/src/cwpack.c"

C_BIN="$CARGO_TARGET_DIR/release/sticky_insert_bug_c"
RS_BIN="$CARGO_TARGET_DIR/release/examples/sticky_insert_ok"

echo "== C oracle (expect BUG CONFIRMED / exit 1) =="
set +e
"$C_BIN"
C_RC=$?
set -e
if [[ "$C_RC" -ne 1 ]]; then
  echo "FAIL: expected C exit 1 (bug present), got $C_RC"
  echo "      (exit 0 would mean upstream fixed insert; update this harness)"
  exit 1
fi

echo
echo "== Rust port (expect OK / exit 0) =="
"$RS_BIN"

echo
echo "sticky-insert differential: C broken, Rust correct — documented OK"
