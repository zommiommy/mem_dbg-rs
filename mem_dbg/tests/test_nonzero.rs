#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::num::*;

#[test]
fn test_nonzero_u8_in_struct() {
    #[derive(MemSize)]
    struct Test {
        value: NonZeroU8,
    }

    let s = Test {
        value: NonZeroU8::new(42).unwrap(),
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<NonZeroU8>());
}

#[test]
fn test_nonzero_integers_in_struct() {
    #[derive(MemSize)]
    struct Test {
        nz_i8: NonZeroI8,
        nz_i16: NonZeroI16,
        nz_i32: NonZeroI32,
        nz_i64: NonZeroI64,
        nz_i128: NonZeroI128,
        nz_isize: NonZeroIsize,
        nz_u8: NonZeroU8,
        nz_u16: NonZeroU16,
        nz_u32: NonZeroU32,
        nz_u64: NonZeroU64,
        nz_u128: NonZeroU128,
        nz_usize: NonZeroUsize,
    }

    let s = Test {
        nz_i8: NonZeroI8::new(1).unwrap(),
        nz_i16: NonZeroI16::new(2).unwrap(),
        nz_i32: NonZeroI32::new(3).unwrap(),
        nz_i64: NonZeroI64::new(4).unwrap(),
        nz_i128: NonZeroI128::new(5).unwrap(),
        nz_isize: NonZeroIsize::new(6).unwrap(),
        nz_u8: NonZeroU8::new(7).unwrap(),
        nz_u16: NonZeroU16::new(8).unwrap(),
        nz_u32: NonZeroU32::new(9).unwrap(),
        nz_u64: NonZeroU64::new(10).unwrap(),
        nz_u128: NonZeroU128::new(11).unwrap(),
        nz_usize: NonZeroUsize::new(12).unwrap(),
    };

    let size = s.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<Test>();

    assert_eq!(size, expected);
}

#[test]
fn test_nonzero_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        nz_u32: NonZeroU32,
        nz_u64: NonZeroU64,
    }

    let s = Test {
        nz_u32: NonZeroU32::new(42).unwrap(),
        nz_u64: NonZeroU64::new(1000).unwrap(),
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
