#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::ops::*;

#[test]
fn test_range_in_struct() {
    #[derive(MemSize)]
    struct Test {
        range: Range<usize>,
    }

    let s = Test { range: 0..10 };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<Range<usize>>());
}

#[test]
fn test_range_from_in_struct() {
    #[derive(MemSize)]
    struct Test {
        range: RangeFrom<usize>,
    }

    let s = Test { range: 5.. };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<RangeFrom<usize>>());
}

#[test]
fn test_range_inclusive_in_struct() {
    #[derive(MemSize)]
    struct Test {
        range: RangeInclusive<usize>,
    }

    let s = Test { range: 0..=10 };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<RangeInclusive<usize>>());
}

#[test]
fn test_range_to_in_struct() {
    #[derive(MemSize)]
    struct Test {
        range: RangeTo<usize>,
    }

    let s = Test { range: ..10 };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<RangeTo<usize>>());
}

#[test]
fn test_range_to_inclusive_in_struct() {
    #[derive(MemSize)]
    struct Test {
        range: RangeToInclusive<usize>,
    }

    let s = Test { range: ..=10 };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<RangeToInclusive<usize>>());
}

#[test]
fn test_all_ranges_in_struct() {
    #[derive(MemSize)]
    struct Test {
        range: Range<usize>,
        range_from: RangeFrom<usize>,
        range_inclusive: RangeInclusive<usize>,
        range_to: RangeTo<usize>,
        range_to_inclusive: RangeToInclusive<usize>,
    }

    let s = Test {
        range: 0..10,
        range_from: 5..,
        range_inclusive: 0..=10,
        range_to: ..10,
        range_to_inclusive: ..=10,
    };

    let size = s.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<Test>();

    assert_eq!(size, expected);
}

#[test]
fn test_ranges_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        range: Range<i32>,
        range_from: RangeFrom<i32>,
        range_inclusive: RangeInclusive<i32>,
    }

    let s = Test {
        range: 1..100,
        range_from: 50..,
        range_inclusive: 0..=99,
    };

    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, std::mem::size_of::<Test>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
