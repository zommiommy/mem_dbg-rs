#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;

mod all_types_helper;
use all_types_helper::run_all_types_test;

#[test]
/// Test mem_dbg on AllTypesStruct with various flags.
fn test_all_types_mem_dbg() {
    // Test with all combinations of depth and flags
    let flags_set = [
        DbgFlags::empty(),
        DbgFlags::default(),
        DbgFlags::FOLLOW_REFS,
        DbgFlags::HUMANIZE,
        DbgFlags::PERCENTAGE,
        DbgFlags::TYPE_NAME,
        DbgFlags::CAPACITY,
        DbgFlags::SEPARATOR,
        DbgFlags::RUST_LAYOUT,
        DbgFlags::COLOR,
        DbgFlags::FOLLOW_RC,
    ];
    for first in flags_set {
        for second in flags_set {
            for third in flags_set {
                let combined_flags = first | second | third;
                let mut output = String::new();
                assert!(
                    run_all_types_test(|all_types| all_types
                        .mem_dbg_on(&mut output, combined_flags)
                        .is_ok()),
                    "mem_dbg_on with flags {combined_flags:?} should succeed",
                );
                for depth in 0..3 {
                    let mut depth_output = String::new();
                    assert!(
                        run_all_types_test(|all_types| all_types
                            .mem_dbg_depth_on(&mut depth_output, depth, combined_flags)
                            .is_ok()),
                        "mem_dbg_depth_on with depth {depth} and flags {combined_flags:?} should succeed",
                    );
                }
            }
        }
    }
}
