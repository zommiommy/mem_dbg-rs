#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::marker::{PhantomData, PhantomPinned};

#[test]
fn test_phantom_data_in_struct() {
    #[derive(MemSize)]
    struct Test<T> {
        phantom: PhantomData<T>,
    }

    let s = Test::<i32> {
        phantom: PhantomData,
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<PhantomData<i32>>());
}

#[test]
fn test_phantom_pinned_in_struct() {
    #[derive(MemSize)]
    struct Test {
        pinned: PhantomPinned,
    }

    let s = Test {
        pinned: PhantomPinned,
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<PhantomPinned>());
}

#[test]
fn test_phantom_types_in_struct() {
    #[derive(MemSize)]
    struct Test<T, U> {
        phantom_data: PhantomData<T>,
        phantom_pinned: PhantomPinned,
        phantom_data2: PhantomData<U>,
    }

    let s = Test::<String, Vec<i32>> {
        phantom_data: PhantomData,
        phantom_pinned: PhantomPinned,
        phantom_data2: PhantomData,
    };

    let size = s.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<Test<String, Vec<i32>>>();

    assert_eq!(size, expected);
}

#[test]
fn test_phantom_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test<T> {
        phantom_data: PhantomData<T>,
        phantom_pinned: PhantomPinned,
    }

    let s = Test::<u64> {
        phantom_data: PhantomData,
        phantom_pinned: PhantomPinned,
    };

    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, std::mem::size_of::<Test<u64>>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
