//! Practical contrast to extra-tests/sticky_insert_bug.c:
//! same encode path, but insert is a no-op after sticky error — buffer stays
//! truncated (no fake "payload" integer for a receiver).
//!
//! Exit 0 = correct (no corrupt append; incomplete map is not silently wrong).
//! Exit 1 = port bug (insert wrote).
//! Exit 2 = setup failed.

use cwpack::{
    cw_pack_boolean, cw_pack_ext, cw_pack_insert, cw_pack_map_size, cw_pack_set_compatibility,
    cw_pack_str, cw_unpack_next, CwPackContext, CwUnpackContext, Error, ItemType,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut buf = [0u8; 64];

    /* Prove compat forbids EXT. */
    {
        let mut pc = CwPackContext::new(&mut buf);
        cw_pack_set_compatibility(&mut pc, true);
        cw_pack_ext(&mut pc, 1, &[0x99], 1);
        if pc.return_code != Error::IllegalCall.code() || pc.len_packed() != 0 {
            eprintln!("setup failed: compat must reject EXT");
            return ExitCode::from(2);
        }
    }
    eprintln!("Rust OK: compatibility forbids EXT (sticky IllegalCall)\n");

    buf.fill(0);
    let n = {
        let mut pc = CwPackContext::new(&mut buf);
        cw_pack_set_compatibility(&mut pc, true);

        eprintln!("App encodes map {{ status: true, payload: <ext> }} (compat ON)");
        cw_pack_map_size(&mut pc, 2);
        cw_pack_str(&mut pc, b"status", 6);
        cw_pack_boolean(&mut pc, true);
        cw_pack_str(&mut pc, b"payload", 7);
        cw_pack_ext(&mut pc, 1, &[0x99], 1);
        eprintln!(
            "  after failed ext: return_code={} packed={}",
            pc.return_code,
            pc.len_packed()
        );
        if pc.return_code != Error::IllegalCall.code() {
            eprintln!("setup failed: expected IllegalCall");
            return ExitCode::from(2);
        }

        let before = pc.len_packed();
        eprintln!("  fallback: cw_pack_insert(\"BUG!\") while rc is sticky error");
        cw_pack_insert(&mut pc, b"BUG!", 4);
        let after = pc.len_packed();
        eprintln!(
            "  after insert: return_code={} packed={} wrote={}",
            pc.return_code,
            after,
            after.saturating_sub(before)
        );
        if after != before {
            eprintln!("Rust FAIL: insert appended despite sticky error");
            return ExitCode::from(1);
        }
        after
    };

    eprint!("  wire ({n} bytes):");
    for b in &buf[..n] {
        eprint!(" {b:02x}");
    }
    eprintln!();

    /* If a sender still ships the truncated buffer, unpack must NOT invent payload=66. */
    eprintln!("\nIf sender ships truncated buffer, receiver unpacks:");
    let mut uc = CwUnpackContext::new(&buf[..n]);
    cw_unpack_next(&mut uc);
    if uc.return_code != 0 || uc.item.type_code != ItemType::Map as i32 {
        eprintln!("setup failed: map header");
        return ExitCode::from(2);
    }
    cw_unpack_next(&mut uc); // "status"
    cw_unpack_next(&mut uc); // true
    eprintln!("  status  = true");
    cw_unpack_next(&mut uc); // "payload"
    cw_unpack_next(&mut uc); // missing value → should error, not fake int
    if uc.return_code == 0 && uc.item.type_code == ItemType::PositiveInteger as i32 {
        eprintln!(
            "Rust FAIL: decoded fake payload={} (corruption slipped through)",
            uc.item.u64
        );
        return ExitCode::from(1);
    }
    eprintln!(
        "  payload = <error rc={}>  ← honest failure (no silent wrong integer)",
        uc.return_code
    );
    eprintln!("\nRust OK: sticky insert did not corrupt user-visible fields");
    ExitCode::SUCCESS
}
