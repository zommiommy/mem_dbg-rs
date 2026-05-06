#![cfg(all(feature = "std", feature = "derive"))]

//! Core wrapper fields should delegate accounting and debug traversal to the
//! payload variant that is actually present.

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
fn active_wrapper_payloads_are_accounted_and_debuggable() {
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
    node.mem_dbg_depth_on(&mut output, 2, DbgFlags::default())
        .unwrap();
    assert!(output.contains("ActiveWrapperFields"));
    assert!(output.contains("reverse"));
    assert!(output.contains("bound"));
    assert!(output.contains("poll"));
    assert!(output.contains("flow_break"));
    assert!(output.contains("flow_continue"));
}

#[test]
fn empty_wrapper_variants_have_no_payload_contribution() {
    let node = EmptyWrapperFields {
        bound: core::ops::Bound::Unbounded,
        poll: core::task::Poll::Pending,
    };

    assert_eq!(
        node.mem_size(SizeFlags::default()),
        core::mem::size_of::<EmptyWrapperFields>()
    );

    let mut output = String::new();
    node.mem_dbg_depth_on(&mut output, 2, DbgFlags::default())
        .unwrap();
    assert!(output.contains("EmptyWrapperFields"));
    assert!(output.contains("bound"));
    assert!(output.contains("poll"));
}

#[test]
fn individual_wrapper_sizes_match_active_payloads() {
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
}
