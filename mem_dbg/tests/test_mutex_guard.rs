//! Test to ensure MutexGuard is handled correctly.
#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;
use std::sync::{Mutex, MutexGuard};

#[test]
/// Test mem_size on a struct containing a MutexGuard.
fn test_mutex_guard() {
    #[derive(MemSize)]
    struct Test<'a> {
        guard: MutexGuard<'a, i32>,
    }

    let mutex_source = Mutex::new(0);

    let s = Test {
        guard: mutex_source.lock().unwrap(),
    };
    let size = s.mem_size(SizeFlags::default());
    let size_of_usize = core::mem::size_of::<usize>();

    assert_eq!(
        size,
        2 * size_of_usize,
        "A MutexGuard is two pointers in size"
    );
}
