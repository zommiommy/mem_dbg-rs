#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;

mod all_types_helper;
use all_types_helper::run_all_types_test;

#[test]
#[cfg_attr(miri, ignore)] // too slow under miri
/// Test mem_size on AllTypesStruct with various flags.
fn test_all_types_mem_size() {
    let flags_set = [
        SizeFlags::empty(),
        SizeFlags::default(),
        SizeFlags::CAPACITY,
        SizeFlags::FOLLOW_REFS,
        SizeFlags::FOLLOW_RCS,
    ];

    for first in flags_set {
        for second in flags_set {
            for third in flags_set {
                let flags = first | second | third;
                let sz = run_all_types_test(|all_types| all_types.mem_size(flags));
                assert!(sz > 0, "mem_size with flags {flags:?} should be > 0");
            }
        }
    }
}
