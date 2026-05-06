#![cfg(feature = "std")]
#![cfg(feature = "derive")]

//! Regression tests for the bugs fixed by the audit pass. Each section is
//! introduced by the bug it covers.

use mem_dbg::*;

#[derive(MemSize, MemDbg)]
struct Inner {
    heavy: Vec<u8>,
}

fn render<T: MemDbg>(value: &T) -> String {
    let mut out = String::new();
    value
        .mem_dbg_on(&mut out, DbgFlags::default())
        .expect("mem_dbg_on");
    out
}

// ---------------------------------------------------------------------------
// Bug 1: `OnceCell<T>` and `OnceLock<T>` `_mem_dbg_rec_on` were no-ops
// because they called `Option::<&T>::_mem_dbg_rec_on` (the empty default)
// instead of unwrapping `self.get()`.
// ---------------------------------------------------------------------------

#[test]
fn oncecell_renders_inner_children() {
    use std::cell::OnceCell;

    let cell: OnceCell<Inner> = OnceCell::new();
    cell.set(Inner {
        heavy: vec![0u8; 64],
    })
    .ok()
    .expect("oncecell set");

    let out = render(&cell);
    assert!(out.contains("OnceCell"), "missing OnceCell line:\n{out}");
    assert!(
        out.contains("heavy"),
        "OnceCell did not recurse into Inner:\n{out}"
    );
}

#[test]
fn oncecell_empty_renders_only_self() {
    use std::cell::OnceCell;

    let cell: OnceCell<Inner> = OnceCell::new();
    let out = render(&cell);
    assert!(out.contains("OnceCell"));
    assert!(!out.contains("heavy"));
}

#[test]
fn oncelock_renders_inner_children() {
    use std::sync::OnceLock;

    let cell: OnceLock<Inner> = OnceLock::new();
    cell.set(Inner {
        heavy: vec![0u8; 64],
    })
    .ok()
    .expect("oncelock set");

    let out = render(&cell);
    assert!(out.contains("OnceLock"), "missing OnceLock line:\n{out}");
    assert!(
        out.contains("heavy"),
        "OnceLock did not recurse into Inner:\n{out}"
    );
}

#[test]
fn oncelock_empty_renders_only_self() {
    use std::sync::OnceLock;

    let cell: OnceLock<Inner> = OnceLock::new();
    let out = render(&cell);
    assert!(out.contains("OnceLock"));
    assert!(!out.contains("heavy"));
}

// ---------------------------------------------------------------------------
// Bug 2: `Mutex<T>::_mem_dbg_rec_on` and `RwLock<T>::_mem_dbg_rec_on`
// dispatched on the `MutexGuard<T>`/`RwLockReadGuard<T>` value, hitting the
// guard's FOLLOW_REFS-gated impl and silently dropping children under
// default flags - while `mem_size_rec` always recursed.
// ---------------------------------------------------------------------------

#[test]
fn mutex_renders_inner_children_under_default_flags() {
    use std::sync::Mutex;

    let m = Mutex::new(Inner {
        heavy: vec![0u8; 64],
    });
    let out = render(&m);
    assert!(out.contains("Mutex"));
    assert!(
        out.contains("heavy"),
        "Mutex did not recurse into Inner under default flags:\n{out}"
    );
}

#[test]
fn rwlock_renders_inner_children_under_default_flags() {
    use std::sync::RwLock;

    let r = RwLock::new(Inner {
        heavy: vec![0u8; 64],
    });
    let out = render(&r);
    assert!(out.contains("RwLock"));
    assert!(
        out.contains("heavy"),
        "RwLock did not recurse into Inner under default flags:\n{out}"
    );
}

// ---------------------------------------------------------------------------
// Bug 3: `Range`/`RangeFrom`/`RangeInclusive`/`RangeTo`/`RangeToInclusive`/
// `Reverse`/`Bound`/`ControlFlow`/`Poll` called `_mem_dbg_rec_on` on inner
// fields instead of `_mem_dbg_depth_on`, losing field labels and miscomputing
// tree corners (the inner field's grandchildren rendered at the wrong depth).
// ---------------------------------------------------------------------------

#[test]
fn range_labels_start_and_end() {
    let r: std::ops::Range<u32> = 1..100;
    let out = render(&r);
    assert!(out.contains("├╴start"), "missing labeled start:\n{out}");
    assert!(out.contains("╰╴end"), "missing labeled end:\n{out}");
}

#[test]
fn range_inclusive_labels_start_and_end() {
    let r: std::ops::RangeInclusive<u32> = 1..=100;
    let out = render(&r);
    assert!(out.contains("├╴start"), "missing labeled start:\n{out}");
    assert!(out.contains("╰╴end"), "missing labeled end:\n{out}");
}

#[test]
fn one_sided_ranges_label_their_only_field() {
    let from: std::ops::RangeFrom<u32> = 5..;
    assert!(render(&from).contains("╰╴start"));

    let to: std::ops::RangeTo<u32> = ..10;
    assert!(render(&to).contains("╰╴end"));

    let to_inclusive: std::ops::RangeToInclusive<u32> = ..=10;
    assert!(render(&to_inclusive).contains("╰╴end"));
}

#[test]
fn reverse_labels_inner_field() {
    let value = core::cmp::Reverse(Box::new(7u32));
    let out = render(&value);
    assert!(out.contains("Reverse"));
    assert!(out.contains("╰╴0"), "missing tuple-style 0 label:\n{out}");
}

#[test]
fn bound_variants_label_payload() {
    let included: std::ops::Bound<Box<u32>> = std::ops::Bound::Included(Box::new(1));
    let excluded: std::ops::Bound<Box<u32>> = std::ops::Bound::Excluded(Box::new(2));
    let unbounded: std::ops::Bound<Box<u32>> = std::ops::Bound::Unbounded;

    assert!(render(&included).contains("╰╴Included"));
    assert!(render(&excluded).contains("╰╴Excluded"));
    let out_unbounded = render(&unbounded);
    assert!(out_unbounded.contains("Bound"));
    assert!(!out_unbounded.contains("Included"));
    assert!(!out_unbounded.contains("Excluded"));
}

#[test]
fn poll_variants_label_payload() {
    let ready: core::task::Poll<Box<u32>> = core::task::Poll::Ready(Box::new(1));
    let pending: core::task::Poll<Box<u32>> = core::task::Poll::Pending;

    assert!(render(&ready).contains("╰╴Ready"));
    let out_pending = render(&pending);
    assert!(!out_pending.contains("Ready"));
}

#[test]
fn control_flow_variants_label_payload() {
    let brk: core::ops::ControlFlow<Box<u32>, Box<u32>> =
        core::ops::ControlFlow::Break(Box::new(1));
    let cont: core::ops::ControlFlow<Box<u32>, Box<u32>> =
        core::ops::ControlFlow::Continue(Box::new(2));

    assert!(render(&brk).contains("╰╴Break"));
    assert!(render(&cont).contains("╰╴Continue"));
}
