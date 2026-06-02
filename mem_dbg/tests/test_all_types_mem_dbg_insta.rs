#![cfg(all(feature = "std", feature = "derive"))]

use insta::{assert_snapshot, with_settings};
use mem_dbg::*;
use regex::Regex;

mod all_types_helper;
use all_types_helper::run_all_types_test;

/// Redact dynamic pointer addresses to make snapshots deterministic.
/// Replaces each unique address with a numbered placeholder (e.g., @ 0x[ADDR_1], → 0x[ADDR_1])
/// so that we can verify that shared references have the same address and different ones don't.
fn redact_addresses(s: &str) -> String {
    use std::collections::HashMap;
    // Capture the prefix and the hex address separately
    let re = Regex::new(r"(@ |→ )(0x[a-fA-F0-9]{8,16})").unwrap();
    let mut addr_map: HashMap<String, usize> = HashMap::new();
    let mut counter = 1;

    re.replace_all(s, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let hex_addr = caps[2].to_string();
        let id = *addr_map.entry(hex_addr).or_insert_with(|| {
            let id = counter;
            counter += 1;
            id
        });
        format!("{}0x[ADDR_{}]", prefix, id)
    })
    .to_string()
}

/// Normalize type-name rendering that changed across Rust toolchains.
fn normalize_type_names(s: String) -> String {
    let mut s = s
        .replace(
            "all_types_helper::AllTypesStruct<'_>",
            "all_types_helper::AllTypesStruct",
        )
        .replace("MutexGuard<'_, ", "MutexGuard<")
        .replace("RwLockReadGuard<'_, ", "RwLockReadGuard<")
        .replace("RwLockWriteGuard<'_, ", "RwLockWriteGuard<");
    // Atomics render as `Atomic<bool>` on some toolchains and `AtomicBool` on
    // others; canonicalize to the legacy `AtomicBool` spelling.
    for (generic, legacy) in [
        ("Atomic<bool>", "AtomicBool"),
        ("Atomic<i8>", "AtomicI8"),
        ("Atomic<i16>", "AtomicI16"),
        ("Atomic<i32>", "AtomicI32"),
        ("Atomic<i64>", "AtomicI64"),
        ("Atomic<isize>", "AtomicIsize"),
        ("Atomic<u8>", "AtomicU8"),
        ("Atomic<u16>", "AtomicU16"),
        ("Atomic<u32>", "AtomicU32"),
        ("Atomic<u64>", "AtomicU64"),
        ("Atomic<usize>", "AtomicUsize"),
    ] {
        s = s.replace(generic, legacy);
    }
    s
}

#[test]
#[cfg_attr(miri, ignore)] // too slow under miri
fn test_all_types_mem_dbg_snapshot() {
    let combinations = vec![
        ("default", DbgFlags::default()),
        ("empty", DbgFlags::empty()),
        ("capacity", DbgFlags::default() | DbgFlags::CAPACITY),
        ("humanize", DbgFlags::default() | DbgFlags::HUMANIZE),
        ("follow_refs", DbgFlags::default() | DbgFlags::FOLLOW_REFS),
    ];

    // Use target-specific snapshots since memory sizes differ by architecture
    // and aarch64 differs between Linux and Darwin.
    let snapshot_suffix = match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "linux") => "aarch64-linux",
        ("aarch64", "macos") => "aarch64-darwin",
        (arch, _) => arch,
    };

    for (name, flags) in combinations {
        let output = run_all_types_test(|all_types| {
            let mut output = String::new();
            all_types
                .mem_dbg_on(&mut output, flags)
                .expect("mem_dbg_on failed");
            output
        });
        let output = normalize_type_names(redact_addresses(&output));
        with_settings!({snapshot_suffix => snapshot_suffix}, {
            assert_snapshot!(name, output);
        });
        for depth in 0..3 {
            let depth_output = run_all_types_test(|all_types| {
                let mut output = String::new();
                all_types
                    .mem_dbg_depth_on(&mut output, depth, flags)
                    .expect("mem_dbg_depth_on failed");
                output
            });
            let depth_output = normalize_type_names(redact_addresses(&depth_output));
            with_settings!({snapshot_suffix => snapshot_suffix}, {
                assert_snapshot!(format!("{name}_depth_{depth}"), depth_output);
            });
        }
    }
}
