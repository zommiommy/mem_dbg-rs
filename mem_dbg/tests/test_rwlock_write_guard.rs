//! Test to ensure RwLockWriteGuard is handled correctly.
#![cfg(all(feature = "std", feature = "derive"))]

use mem_dbg::*;
use std::sync::{RwLock, RwLockWriteGuard};

#[test]
/// Test mem_size on a struct containing a RwLockWriteGuard.
fn test_rwlock_write_guard() {
    #[derive(MemSize)]
    struct Test<'a> {
        guard: RwLockWriteGuard<'a, String>,
    }

    let rwlock_source = RwLock::new("test".to_string());

    let s = Test {
        guard: rwlock_source.write().unwrap(),
    };
    let size = s.mem_size(SizeFlags::default());
    let size_of_usize = core::mem::size_of::<usize>();

    assert_eq!(
        size,
        2 * size_of_usize,
        "A RwLockWriteGuard is two pointers in size"
    );
}

#[test]
fn test_rwlock_write_guard_follow_refs_counts_target_header() {
    let rwlock_source = RwLock::new(vec![1_u8, 2, 3]);
    let mut guard = rwlock_source.write().unwrap();
    guard.push(4);

    assert_eq!(
        guard.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of_val(&guard) + core::mem::size_of::<Vec<u8>>() + 4
    );
}
