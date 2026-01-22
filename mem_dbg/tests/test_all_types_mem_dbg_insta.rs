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
    let re = Regex::new(r"(@ |→ )(0x[a-fA-F0-9]{16})").unwrap();
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

    // Use architecture-specific snapshots since memory sizes differ
    // (e.g., GROUP_WIDTH is 16 on x86_64 with SSE2, 8 on ARM64)
    let arch = std::env::consts::ARCH;

    for (name, flags) in combinations {
        let output = run_all_types_test(|all_types| {
            let mut output = String::new();
            all_types
                .mem_dbg_on(&mut output, flags)
                .expect("mem_dbg_on failed");
            output
        });
        let output = redact_addresses(&output);
        with_settings!({snapshot_suffix => arch}, {
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
            let depth_output = redact_addresses(&depth_output);
            with_settings!({snapshot_suffix => arch}, {
                assert_snapshot!(format!("{name}_depth_{depth}"), depth_output);
            });
        }
    }
}
