#![cfg(feature = "std")]

//! Followed-cycle recursion is a runtime concern: `Rc`, `Arc`, and references
//! may form cycles once follow flags are enabled. These tests make sure visited
//! pointer tracking stops `MemSize` and `MemDbg` from revisiting the cycle.

use mem_dbg::*;

const MAX_CYCLE_VISITS: usize = 8;

struct CountingRcCycle {
    size_visits: std::rc::Rc<std::cell::Cell<usize>>,
    dbg_visits: std::rc::Rc<std::cell::Cell<usize>>,
    next: std::cell::OnceCell<std::rc::Rc<CountingRcCycle>>,
}

impl FlatType for CountingRcCycle {
    type Flat = False;
}

impl MemSize for CountingRcCycle {
    fn mem_size_rec(
        &self,
        flags: SizeFlags,
        refs: &mut mem_dbg::HashMap<usize, RefRecord>,
    ) -> usize {
        let visits = self.size_visits.get() + 1;
        self.size_visits.set(visits);
        assert!(visits <= MAX_CYCLE_VISITS);

        core::mem::size_of::<Self>()
            + self.next.get().map_or(0, |next| {
                <std::rc::Rc<CountingRcCycle> as MemSize>::mem_size_rec(next, flags, refs)
                    - core::mem::size_of::<std::rc::Rc<CountingRcCycle>>()
            })
    }
}

impl MemDbgImpl for CountingRcCycle {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        _is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut mem_dbg::HashSet<usize>,
    ) -> core::fmt::Result {
        let visits = self.dbg_visits.get() + 1;
        self.dbg_visits.set(visits);
        assert!(visits <= MAX_CYCLE_VISITS);

        if let Some(next) = self.next.get() {
            <std::rc::Rc<CountingRcCycle> as MemDbgImpl>::_mem_dbg_depth_on(
                next,
                writer,
                total_size,
                max_depth,
                prefix,
                Some("next"),
                true,
                core::mem::size_of::<std::rc::Rc<CountingRcCycle>>(),
                flags,
                dbg_refs,
            )?;
        }
        Ok(())
    }
}

struct CountingArcCycle {
    size_visits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    dbg_visits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    next: std::sync::OnceLock<std::sync::Arc<CountingArcCycle>>,
}

impl FlatType for CountingArcCycle {
    type Flat = False;
}

impl MemSize for CountingArcCycle {
    fn mem_size_rec(
        &self,
        flags: SizeFlags,
        refs: &mut mem_dbg::HashMap<usize, RefRecord>,
    ) -> usize {
        let visits = self
            .size_visits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        assert!(visits <= MAX_CYCLE_VISITS);

        core::mem::size_of::<Self>()
            + self.next.get().map_or(0, |next| {
                <std::sync::Arc<CountingArcCycle> as MemSize>::mem_size_rec(next, flags, refs)
                    - core::mem::size_of::<std::sync::Arc<CountingArcCycle>>()
            })
    }
}

impl MemDbgImpl for CountingArcCycle {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        _is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut mem_dbg::HashSet<usize>,
    ) -> core::fmt::Result {
        let visits = self
            .dbg_visits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        assert!(visits <= MAX_CYCLE_VISITS);

        if let Some(next) = self.next.get() {
            <std::sync::Arc<CountingArcCycle> as MemDbgImpl>::_mem_dbg_depth_on(
                next,
                writer,
                total_size,
                max_depth,
                prefix,
                Some("next"),
                true,
                core::mem::size_of::<std::sync::Arc<CountingArcCycle>>(),
                flags,
                dbg_refs,
            )?;
        }
        Ok(())
    }
}

struct CountingRefCycle {
    size_visits: std::cell::Cell<usize>,
    dbg_visits: std::cell::Cell<usize>,
    next: std::cell::OnceCell<&'static CountingRefCycle>,
}

impl FlatType for CountingRefCycle {
    type Flat = False;
}

impl MemSize for CountingRefCycle {
    fn mem_size_rec(
        &self,
        flags: SizeFlags,
        refs: &mut mem_dbg::HashMap<usize, RefRecord>,
    ) -> usize {
        let visits = self.size_visits.get() + 1;
        self.size_visits.set(visits);
        assert!(visits <= MAX_CYCLE_VISITS);

        core::mem::size_of::<Self>()
            + self.next.get().map_or(0, |next| {
                <&CountingRefCycle as MemSize>::mem_size_rec(next, flags, refs)
                    - core::mem::size_of::<&CountingRefCycle>()
            })
    }
}

impl MemDbgImpl for CountingRefCycle {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        _is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut mem_dbg::HashSet<usize>,
    ) -> core::fmt::Result {
        let visits = self.dbg_visits.get() + 1;
        self.dbg_visits.set(visits);
        assert!(visits <= MAX_CYCLE_VISITS);

        if let Some(next) = self.next.get() {
            <&CountingRefCycle as MemDbgImpl>::_mem_dbg_depth_on(
                next,
                writer,
                total_size,
                max_depth,
                prefix,
                Some("next"),
                true,
                core::mem::size_of::<&CountingRefCycle>(),
                flags,
                dbg_refs,
            )?;
        }
        Ok(())
    }
}

// These tests intentionally build ownership cycles (a strong `Rc`/`Arc` cycle,
// and a `Box::leak`ed `&'static` self-reference) to exercise cycle handling, so
// they leak by construction. Miri's leak checker correctly objects, and the
// logic is plain safe Rust covered by the other targets, so skip them on Miri.
#[cfg_attr(miri, ignore = "intentional ownership cycle leaks")]
#[test]
fn followed_rc_cycle_should_not_recurse_forever() {
    let size_visits = std::rc::Rc::new(std::cell::Cell::new(0));
    let dbg_visits = std::rc::Rc::new(std::cell::Cell::new(0));
    let root = std::rc::Rc::new(CountingRcCycle {
        size_visits: size_visits.clone(),
        dbg_visits: dbg_visits.clone(),
        next: std::cell::OnceCell::new(),
    });
    assert!(root.next.set(root.clone()).is_ok());

    let size = root.mem_size(SizeFlags::FOLLOW_RCS);
    assert!(size >= core::mem::size_of::<std::rc::Rc<CountingRcCycle>>());
    assert_eq!(size_visits.get(), 1);

    let mut output = String::new();
    root.mem_dbg_depth_on(&mut output, 2, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .unwrap();
    assert!(output.contains("CountingRcCycle"));
    assert!(output.contains("→ 0x"));
    assert_eq!(dbg_visits.get(), 1);
}

#[cfg_attr(miri, ignore = "intentional ownership cycle leaks")]
#[test]
fn followed_arc_cycle_should_not_recurse_forever() {
    let size_visits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let dbg_visits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let root = std::sync::Arc::new(CountingArcCycle {
        size_visits: size_visits.clone(),
        dbg_visits: dbg_visits.clone(),
        next: std::sync::OnceLock::new(),
    });
    assert!(root.next.set(root.clone()).is_ok());

    let size = root.mem_size(SizeFlags::FOLLOW_RCS);
    assert!(size >= core::mem::size_of::<std::sync::Arc<CountingArcCycle>>());
    assert_eq!(size_visits.load(std::sync::atomic::Ordering::Relaxed), 1);

    let mut output = String::new();
    root.mem_dbg_depth_on(&mut output, 2, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .unwrap();
    assert!(output.contains("CountingArcCycle"));
    assert!(output.contains("→ 0x"));
    assert_eq!(dbg_visits.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[cfg_attr(miri, ignore = "intentional ownership cycle leaks")]
#[test]
fn followed_reference_cycle_should_not_recurse_forever() {
    let node: &'static CountingRefCycle = Box::leak(Box::new(CountingRefCycle {
        size_visits: std::cell::Cell::new(0),
        dbg_visits: std::cell::Cell::new(0),
        next: std::cell::OnceCell::new(),
    }));
    assert!(node.next.set(node).is_ok());

    let size = <&CountingRefCycle as MemSize>::mem_size(&node, SizeFlags::FOLLOW_REFS);
    assert!(size >= core::mem::size_of::<&CountingRefCycle>());
    assert_eq!(node.size_visits.get(), 1);

    let mut output = String::new();
    <&CountingRefCycle as MemDbg>::mem_dbg_depth_on(
        &node,
        &mut output,
        2,
        DbgFlags::default() | DbgFlags::FOLLOW_REFS,
    )
    .unwrap();
    assert!(output.contains("CountingRefCycle"));
    assert!(output.contains("→ 0x"));
    assert_eq!(node.dbg_visits.get(), 1);
}
