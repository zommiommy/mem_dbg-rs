#![cfg(feature = "std")]

//! Precise size assertions for `Mutex<T>` and `RwLock<T>`. The implementation
//! locks the inner state to compute `size_of::<Self>() - size_of::<T>() +
//! T::mem_size_rec(inner)`, so the reported size is the lock's stack
//! footprint plus the recursive size of the protected value.

use mem_dbg::*;
use std::sync::{Mutex, RwLock};

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
fn test_mutex_mem_size_while_locked_reports_shell_only() {
    let m = Mutex::new(String::from("hello"));
    let _guard = m.lock().expect("lock");

    assert_eq!(
        m.mem_size(SizeFlags::default()),
        core::mem::size_of::<Mutex<String>>()
    );
    assert_eq!(
        m.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<Mutex<String>>()
    );
}

#[test]
fn test_rwlock_mem_size_while_write_locked_reports_shell_only() {
    let r = RwLock::new(String::from("hello"));
    let _guard = r.write().expect("write lock");

    assert_eq!(
        r.mem_size(SizeFlags::default()),
        core::mem::size_of::<RwLock<String>>()
    );
    assert_eq!(
        r.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<RwLock<String>>()
    );
}

#[test]
fn test_mutex_mem_dbg_while_locked_marks_locked() {
    let m = Mutex::new(vec![1_u8, 2, 3]);
    let _guard = m.lock().expect("lock");
    let mut output = String::new();

    m.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on");

    assert!(output.contains("<locked>"));
}

#[test]
fn test_rwlock_mem_dbg_while_write_locked_marks_locked() {
    let r = RwLock::new(vec![1_u8, 2, 3]);
    let _guard = r.write().expect("write lock");
    let mut output = String::new();

    r.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on");

    assert!(output.contains("<locked>"));
}
