#!/usr/bin/env bash
# JSON fixtures → ops → MessagePack via C CWPack and Rust cwpack-rs → byte-compare.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CWPACK_SRC="${CWPACK_SRC:-$ROOT/../CWPack}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
OUT="$ROOT/extra-tests/out"
mkdir -p "$OUT"

cd "$ROOT"
cargo build --release --example ops_pack

clang -O3 -I "$CWPACK_SRC/src" \
  -o "$CARGO_TARGET_DIR/release/ops_pack_c" \
  extra-tests/ops_pack_c.c "$CWPACK_SRC/src/cwpack.c"

C_BIN="$CARGO_TARGET_DIR/release/ops_pack_c"
R_BIN="$CARGO_TARGET_DIR/release/examples/ops_pack"

FILES=(
  extra-tests/fixtures/mixed_types.json
  extra-tests/fixtures/nested_config.json
  extra-tests/fixtures/events.json
  extra-tests/fixtures/cities_numeric_5k.json
  extra-tests/countries.json
)

# Optional large (slow ops gen): cities.json / one IP block
if [[ "${INCLUDE_LARGE:-0}" == "1" ]]; then
  FILES+=(extra-tests/cities.json)
  if [[ -f extra-tests/country-ip-blocks/country/us/aggregated.json ]]; then
    FILES+=(extra-tests/country-ip-blocks/country/us/aggregated.json)
  fi
fi

fail=0
for f in "${FILES[@]}"; do
  base=$(basename "$f" .json)
  echo "== $f"
  python3 extra-tests/json_to_ops.py "$f" -o "$OUT/$base.ops"
  "$C_BIN" "$OUT/$base.ops" > "$OUT/$base.c.mp"
  "$R_BIN" < "$OUT/$base.ops" > "$OUT/$base.rs.mp"
  if cmp -s "$OUT/$base.c.mp" "$OUT/$base.rs.mp"; then
    bytes=$(wc -c < "$OUT/$base.c.mp" | tr -d ' ')
    echo "OK  bytes=$bytes"
  else
    echo "FAIL byte mismatch"
    ls -la "$OUT/$base.c.mp" "$OUT/$base.rs.mp"
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo "differential JSON→msgpack: FAILED"
  exit 1
fi
echo "differential JSON→msgpack: all OK"
