#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use core::cell::UnsafeCell;
use mem_dbg::*;

#[test]
fn test_minimal_unsafecell_in_struct() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        field: UnsafeCell<i32>,
    }

    let s = Test {
        field: UnsafeCell::new(42),
    };
    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, 4)
}
