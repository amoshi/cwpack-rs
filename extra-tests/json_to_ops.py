#!/usr/bin/env python3
"""JSON → deterministic op stream for CWPack packers (C and Rust).

Object keys are sorted for stable MessagePack output.
Numbers: ints → I64/U64; integer-valued floats in range → int; else F64BITS (native IEEE bits).
"""
from __future__ import annotations

import argparse
import json
import math
import struct
import sys
from pathlib import Path
from typing import Any, BinaryIO


def emit_str(out: BinaryIO, s: str) -> None:
    b = s.encode("utf-8")
    out.write(f"STR {len(b)}\n".encode("ascii"))
    out.write(b)
    out.write(b"\n")


def emit(out: BinaryIO, v: Any) -> None:
    if v is None:
        out.write(b"NIL\n")
        return
    if isinstance(v, bool):
        out.write(f"BOOL {1 if v else 0}\n".encode("ascii"))
        return
    if isinstance(v, int) and not isinstance(v, bool):
        if v >= 0:
            out.write(f"U64 {v}\n".encode("ascii"))
        else:
            out.write(f"I64 {v}\n".encode("ascii"))
        return
    if isinstance(v, float):
        if math.isfinite(v) and v.is_integer() and -(2**63) <= v < 2**64:
            iv = int(v)
            if iv >= 0:
                out.write(f"U64 {iv}\n".encode("ascii"))
            else:
                out.write(f"I64 {iv}\n".encode("ascii"))
        else:
            bits = struct.unpack("Q", struct.pack("d", v))[0]
            out.write(f"F64BITS {bits}\n".encode("ascii"))
        return
    if isinstance(v, str):
        emit_str(out, v)
        return
    if isinstance(v, list):
        out.write(f"ARR {len(v)}\n".encode("ascii"))
        for item in v:
            emit(out, item)
        return
    if isinstance(v, dict):
        keys = sorted(v.keys())
        out.write(f"MAP {len(keys)}\n".encode("ascii"))
        for k in keys:
            emit_str(out, k)
            emit(out, v[k])
        return
    raise TypeError(f"unsupported JSON type: {type(v)}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("json_path")
    ap.add_argument("-o", "--output", help="ops file (default stdout)")
    args = ap.parse_args()
    data = json.loads(Path(args.json_path).read_text(encoding="utf-8"))
    if args.output:
        with open(args.output, "wb") as out:
            emit(out, data)
    else:
        emit(sys.stdout.buffer, data)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
