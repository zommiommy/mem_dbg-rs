#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;

mod all_types_helper;
use all_types_helper::run_all_types_test;

#[test]
/// Test mem_size on AllTypesStruct with various flags.
fn test_all_types_mem_size() {
    for flags in [
        SizeFlags::empty(),
        SizeFlags::default(),
        SizeFlags::CAPACITY,
        SizeFlags::FOLLOW_REFS,
        SizeFlags::CAPACITY | SizeFlags::FOLLOW_REFS,
        SizeFlags::FOLLOW_RC,
        SizeFlags::CAPACITY | SizeFlags::FOLLOW_RC,
    ] {
        let sz = run_all_types_test(|all_types| all_types.mem_size(flags));
        assert!(sz > 0, "mem_size with flags {:?} should be > 0", flags);
    }
}
