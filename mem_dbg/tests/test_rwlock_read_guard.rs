//! Test to ensure RwLockReadGuard is handled correctly.
#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;
use std::sync::{RwLock, RwLockReadGuard};

#[test]
/// Test mem_size on a struct containing a RwLockReadGuard.
fn test_rwlock_read_guard() {
    #[derive(MemSize)]
    struct Test<'a> {
        guard: RwLockReadGuard<'a, String>,
    }

    let rwlock_source = RwLock::new("test".to_string());

    let s = Test {
        guard: rwlock_source.read().unwrap(),
    };
    let size = s.mem_size(SizeFlags::default());
    let size_of_usize = core::mem::size_of::<usize>();

    assert_eq!(
        size,
        2 * size_of_usize,
        "A RwLockReadGuard is two pointers in size"
    );
}
