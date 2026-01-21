#![cfg(all(feature = "std", feature = "derive"))]

use insta::{assert_snapshot, with_settings};
use mem_dbg::*;

mod all_types_helper;
use all_types_helper::run_all_types_test;

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
            with_settings!({snapshot_suffix => arch}, {
                assert_snapshot!(format!("{name}_depth_{depth}"), depth_output);
            });
        }
    }
}
