#![cfg(all(feature = "parking_lot", feature = "derive"))]

//! Regression test for the `mem_dbg` tree output of parking_lot locks.
//!
//! Mirrors the std "Bug 2" case in `test_audit_regressions.rs`: under default
//! flags, `Mutex<T>::_mem_dbg_rec_on` must recurse into `T` directly rather
//! than dispatching through the `MutexGuard<T>` impl (which is FOLLOW_REFS
//! gated and would silently drop the children). `mem_size` always recurses, so
//! without this the tree disagrees with the reported total.

use mem_dbg::*;
use parking_lot::{Mutex, RwLock};

#[derive(MemSize, MemDbg)]
struct Inner {
    heavy: Vec<u8>,
}

#[test]
fn parking_lot_mutex_renders_inner_children_under_default_flags() {
    let m = Mutex::new(Inner {
        heavy: vec![0u8; 64],
    });

    let mut out = String::new();
    m.mem_dbg_on(&mut out, DbgFlags::default())
        .expect("mem_dbg_on");

    // The `heavy` field label only ever appears on the recursed child line,
    // never in the `Mutex<..>` type name, so this is a faithful signal that
    // the inner value was traversed.
    assert!(
        out.contains("heavy"),
        "Mutex did not recurse into Inner under default flags:\n{out}"
    );
}

#[test]
fn parking_lot_rwlock_renders_inner_children_under_default_flags() {
    let r = RwLock::new(Inner {
        heavy: vec![0u8; 64],
    });

    let mut out = String::new();
    r.mem_dbg_on(&mut out, DbgFlags::default())
        .expect("mem_dbg_on");

    assert!(
        out.contains("heavy"),
        "RwLock did not recurse into Inner under default flags:\n{out}"
    );
}
