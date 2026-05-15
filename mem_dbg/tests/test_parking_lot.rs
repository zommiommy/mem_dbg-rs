#![cfg(feature = "parking_lot")]

//! Precise size assertions for parking_lot's `Mutex<T>` / `RwLock<T>` and
//! their guards. Mirrors `test_locks.rs` and the std guard tests.

use mem_dbg::*;
use parking_lot::{Mutex, RwLock};

#[test]
fn test_mutex_flat_payload() {
    let m = Mutex::new(42_u64);
    assert_eq!(
        m.mem_size(SizeFlags::default()),
        core::mem::size_of::<Mutex<u64>>()
    );
}

#[test]
fn test_mutex_string_payload() {
    let s = String::from("hello");
    let len = s.len();
    let cap = s.capacity();
    let m = Mutex::new(s);
    assert_eq!(
        m.mem_size(SizeFlags::default()),
        core::mem::size_of::<Mutex<String>>() + len
    );
    assert_eq!(
        m.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<Mutex<String>>() + cap
    );
}

#[test]
fn test_mutex_vec_payload() {
    let m = Mutex::new(vec![1u32, 2, 3]);
    let len = 3;
    assert_eq!(
        m.mem_size(SizeFlags::default()),
        core::mem::size_of::<Mutex<Vec<u32>>>() + len * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_rwlock_flat_payload() {
    let r = RwLock::new(42_u64);
    assert_eq!(
        r.mem_size(SizeFlags::default()),
        core::mem::size_of::<RwLock<u64>>()
    );
}

#[test]
fn test_rwlock_string_payload() {
    let s = String::from("hello");
    let len = s.len();
    let cap = s.capacity();
    let r = RwLock::new(s);
    assert_eq!(
        r.mem_size(SizeFlags::default()),
        core::mem::size_of::<RwLock<String>>() + len
    );
    assert_eq!(
        r.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<RwLock<String>>() + cap
    );
}

#[test]
fn test_rwlock_vec_payload() {
    let r = RwLock::new(vec![1u32, 2, 3]);
    let len = 3;
    assert_eq!(
        r.mem_size(SizeFlags::default()),
        core::mem::size_of::<RwLock<Vec<u32>>>() + len * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_mutex_guard() {
    let m = Mutex::new(0_i32);
    let g = m.lock();
    assert_eq!(g.mem_size(SizeFlags::default()), core::mem::size_of_val(&g));
}

#[test]
fn test_rwlock_read_guard() {
    let r = RwLock::new(0_i32);
    let g = r.read();
    assert_eq!(g.mem_size(SizeFlags::default()), core::mem::size_of_val(&g));
}

#[test]
fn test_rwlock_write_guard() {
    let r = RwLock::new(0_i32);
    let g = r.write();
    assert_eq!(g.mem_size(SizeFlags::default()), core::mem::size_of_val(&g));
}
