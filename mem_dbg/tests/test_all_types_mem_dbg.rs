#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;

mod all_types_helper;
use all_types_helper::run_all_types_test;

#[test]
/// Test mem_dbg on AllTypesStruct with various flags.
fn test_all_types_mem_dbg() {
    // Test with all combinations of depth and flags
    for flags in [
        DbgFlags::default(),
        DbgFlags::CAPACITY,
        DbgFlags::COLOR,
        DbgFlags::HUMANIZE,
        DbgFlags::CAPACITY | DbgFlags::COLOR,
    ] {
        dbg!(flags);
        assert!(
            run_all_types_test(|all_types| all_types.mem_dbg(flags).is_ok()),
            "mem_dbg with flags {:?} should succeed",
            flags
        );
        let mut output = String::new();
        assert!(
            run_all_types_test(|all_types| all_types.mem_dbg_on(&mut output, flags).is_ok()),
            "mem_dbg_on with flags {:?} should succeed",
            flags
        );
        for depth in 0..5 {
            assert!(
                run_all_types_test(|all_types| all_types.mem_dbg_depth(depth, flags).is_ok()),
                "mem_dbg_depth with depth {} and flags {:?} should succeed",
                depth,
                flags
            );
            let mut depth_output = String::new();
            assert!(
                run_all_types_test(|all_types| all_types
                    .mem_dbg_depth_on(&mut depth_output, depth, flags)
                    .is_ok()),
                "mem_dbg_depth_on with depth {} and flags {:?} should succeed",
                depth,
                flags
            );
        }
    }
}
