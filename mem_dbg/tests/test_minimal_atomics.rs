#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::sync::atomic::*;

#[test]
fn test_atomic_bool_in_struct() {
    #[derive(MemSize)]
    struct Test {
        atomic: AtomicBool,
    }

    let s = Test {
        atomic: AtomicBool::new(true),
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<AtomicBool>());
}

#[test]
fn test_atomic_integers_in_struct() {
    #[derive(MemSize)]
    struct Test {
        atomic_i8: AtomicI8,
        atomic_i16: AtomicI16,
        atomic_i32: AtomicI32,
        atomic_i64: AtomicI64,
        atomic_isize: AtomicIsize,
        atomic_u8: AtomicU8,
        atomic_u16: AtomicU16,
        atomic_u32: AtomicU32,
        atomic_u64: AtomicU64,
        atomic_usize: AtomicUsize,
    }

    let s = Test {
        atomic_i8: AtomicI8::new(1),
        atomic_i16: AtomicI16::new(2),
        atomic_i32: AtomicI32::new(3),
        atomic_i64: AtomicI64::new(4),
        atomic_isize: AtomicIsize::new(5),
        atomic_u8: AtomicU8::new(6),
        atomic_u16: AtomicU16::new(7),
        atomic_u32: AtomicU32::new(8),
        atomic_u64: AtomicU64::new(9),
        atomic_usize: AtomicUsize::new(10),
    };

    let size = s.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<Test>();

    assert_eq!(size, expected);
}

#[test]
fn test_atomic_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        atomic_bool: AtomicBool,
        atomic_u32: AtomicU32,
    }

    let s = Test {
        atomic_bool: AtomicBool::new(false),
        atomic_u32: AtomicU32::new(42),
    };

    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, std::mem::size_of::<Test>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());
}
