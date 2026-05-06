#![cfg(all(feature = "std", feature = "derive"))]

//! Core wrapper fields should delegate accounting and debug traversal to the
//! payload variant that is actually present.

use anyhow::Context;
use mem_dbg::*;

fn boxed_usize_payload_size() -> usize {
    core::mem::size_of::<usize>()
}

#[derive(MemSize, MemDbg)]
#[mem_size(rec)]
struct ActiveWrapperFields {
    reverse: core::cmp::Reverse<Box<usize>>,
    bound: core::ops::Bound<Box<usize>>,
    poll: core::task::Poll<Box<usize>>,
    flow_break: core::ops::ControlFlow<Box<usize>, Box<usize>>,
    flow_continue: core::ops::ControlFlow<Box<usize>, Box<usize>>,
}

#[derive(MemSize, MemDbg)]
#[mem_size(rec)]
struct EmptyWrapperFields {
    bound: core::ops::Bound<Box<usize>>,
    poll: core::task::Poll<Box<usize>>,
}

#[test]
fn test_active_wrapper_payloads() -> anyhow::Result<()> {
    let node = ActiveWrapperFields {
        reverse: core::cmp::Reverse(Box::new(1)),
        bound: core::ops::Bound::Included(Box::new(2)),
        poll: core::task::Poll::Ready(Box::new(3)),
        flow_break: core::ops::ControlFlow::Break(Box::new(4)),
        flow_continue: core::ops::ControlFlow::Continue(Box::new(5)),
    };

    assert_eq!(
        node.mem_size(SizeFlags::default()),
        core::mem::size_of::<ActiveWrapperFields>() + 5 * boxed_usize_payload_size()
    );

    let mut output = String::new();
    node.mem_dbg_depth_on(&mut output, 2, DbgFlags::default())?;
    assert!(output.contains("ActiveWrapperFields"));
    assert!(output.contains("reverse"));
    assert!(output.contains("bound"));
    assert!(output.contains("poll"));
    assert!(output.contains("flow_break"));
    assert!(output.contains("flow_continue"));

    Ok(())
}

#[test]
fn test_empty_wrapper_variants() -> anyhow::Result<()> {
    let node = EmptyWrapperFields {
        bound: core::ops::Bound::Unbounded,
        poll: core::task::Poll::Pending,
    };

    assert_eq!(
        node.mem_size(SizeFlags::default()),
        core::mem::size_of::<EmptyWrapperFields>()
    );

    let mut output = String::new();
    node.mem_dbg_depth_on(&mut output, 2, DbgFlags::default())?;
    assert!(output.contains("EmptyWrapperFields"));
    assert!(output.contains("bound"));
    assert!(output.contains("poll"));

    Ok(())
}

#[test]
fn test_individual_wrapper_sizes() -> anyhow::Result<()> {
    let reverse = core::cmp::Reverse(Box::new(1usize));
    assert_eq!(
        reverse.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::cmp::Reverse<Box<usize>>>() + boxed_usize_payload_size()
    );

    let included = core::ops::Bound::Included(Box::new(1usize));
    let excluded = core::ops::Bound::Excluded(Box::new(1usize));
    let unbounded: core::ops::Bound<Box<usize>> = core::ops::Bound::Unbounded;
    assert_eq!(
        included.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::Bound<Box<usize>>>() + boxed_usize_payload_size()
    );
    assert_eq!(
        excluded.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::Bound<Box<usize>>>() + boxed_usize_payload_size()
    );
    assert_eq!(
        unbounded.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::Bound<Box<usize>>>()
    );

    let ready = core::task::Poll::Ready(Box::new(1usize));
    let pending: core::task::Poll<Box<usize>> = core::task::Poll::Pending;
    assert_eq!(
        ready.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::task::Poll<Box<usize>>>() + boxed_usize_payload_size()
    );
    assert_eq!(
        pending.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::task::Poll<Box<usize>>>()
    );

    let flow_break: core::ops::ControlFlow<Box<usize>, Box<usize>> =
        core::ops::ControlFlow::Break(Box::new(1usize));
    let flow_continue: core::ops::ControlFlow<Box<usize>, Box<usize>> =
        core::ops::ControlFlow::Continue(Box::new(1usize));
    assert_eq!(
        flow_break.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::ControlFlow<Box<usize>, Box<usize>>>()
            + boxed_usize_payload_size()
    );
    assert_eq!(
        flow_continue.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::ControlFlow<Box<usize>, Box<usize>>>()
            + boxed_usize_payload_size()
    );

    Ok(())
}

/// Niche-optimised payloads must not be double-counted: `mem_size` has to
/// match the compiler-computed `size_of::<Self>()` for every variant, including
/// when the discriminant is folded into the payload's invalid bit pattern. See
/// `test_mem_size::test_exotic` for the equivalent `Option<&T>` regression.
#[test]
fn test_niche_optimized_wrappers() -> anyhow::Result<()> {
    use core::num::NonZeroU32;

    let leaked: &'static u8 = Box::leak(Box::new(7u8));

    // `Poll` has two variants and a single niche from `&u8` is enough to
    // encode `Pending`, so the layout collapses to a single pointer.
    assert_eq!(
        core::mem::size_of::<core::task::Poll<&u8>>(),
        core::mem::size_of::<&u8>(),
    );
    let pending: core::task::Poll<&u8> = core::task::Poll::Pending;
    let ready: core::task::Poll<&u8> = core::task::Poll::Ready(leaked);
    assert_eq!(
        pending.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::task::Poll<&u8>>(),
    );
    assert_eq!(
        ready.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::task::Poll<&u8>>(),
    );

    // `ControlFlow<&u8, ()>` likewise collapses: `Continue(())` is a ZST so
    // the null pointer encodes it.
    assert_eq!(
        core::mem::size_of::<core::ops::ControlFlow<&u8, ()>>(),
        core::mem::size_of::<&u8>(),
    );
    let flow_break: core::ops::ControlFlow<&u8, ()> = core::ops::ControlFlow::Break(leaked);
    let flow_continue: core::ops::ControlFlow<&u8, ()> = core::ops::ControlFlow::Continue(());
    assert_eq!(
        flow_break.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::ControlFlow<&u8, ()>>(),
    );
    assert_eq!(
        flow_continue.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::ops::ControlFlow<&u8, ()>>(),
    );

    // `Reverse` is `repr(transparent)` so it inherits the niche of its
    // payload.
    assert_eq!(
        core::mem::size_of::<core::cmp::Reverse<&u8>>(),
        core::mem::size_of::<&u8>(),
    );
    let reverse = core::cmp::Reverse(leaked);
    assert_eq!(
        reverse.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::cmp::Reverse<&u8>>(),
    );

    // `NonZeroU32` carries a single niche; `Option`/`Poll` collapse to four
    // bytes while `Bound` (three variants) keeps a separate tag. In every
    // case the formula must agree with `size_of::<Self>()`.
    let nz = NonZeroU32::new(5).context("5 is non-zero")?;
    assert_eq!(core::mem::size_of::<Option<NonZeroU32>>(), 4);
    assert_eq!(core::mem::size_of::<core::task::Poll<NonZeroU32>>(), 4);
    let poll_nz_ready = core::task::Poll::<NonZeroU32>::Ready(nz);
    let poll_nz_pending: core::task::Poll<NonZeroU32> = core::task::Poll::Pending;
    assert_eq!(
        poll_nz_ready.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::task::Poll<NonZeroU32>>(),
    );
    assert_eq!(
        poll_nz_pending.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::task::Poll<NonZeroU32>>(),
    );

    let bound_nz_included = core::ops::Bound::<NonZeroU32>::Included(nz);
    let bound_nz_excluded = core::ops::Bound::<NonZeroU32>::Excluded(nz);
    let bound_nz_unbounded: core::ops::Bound<NonZeroU32> = core::ops::Bound::Unbounded;
    let bound_nz_size = core::mem::size_of::<core::ops::Bound<NonZeroU32>>();
    assert_eq!(
        bound_nz_included.mem_size(SizeFlags::default()),
        bound_nz_size
    );
    assert_eq!(
        bound_nz_excluded.mem_size(SizeFlags::default()),
        bound_nz_size
    );
    assert_eq!(
        bound_nz_unbounded.mem_size(SizeFlags::default()),
        bound_nz_size
    );

    Ok(())
}
