#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use core::cell::UnsafeCell;
use mem_dbg::*;
use std::cell::Cell;

#[test]
fn test_unsafe_cell_in_struct() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        field: UnsafeCell<i32>,
    }

    let s = Test {
        field: UnsafeCell::new(42),
    };
    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, 4);

    // This is just to let miri test the unsafe block
    let _ = s.mem_dbg(DbgFlags::default());
}

#[test]
fn test_cell_in_struct() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        field: Cell<i32>,
    }

    let s = Test {
        field: Cell::new(42),
    };
    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, 4);

    // This is just to let miri test the unsafe block
    let _ = s.mem_dbg(DbgFlags::default());
}
