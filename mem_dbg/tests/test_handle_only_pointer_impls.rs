#![cfg(all(feature = "std", feature = "derive"))]

//! Handle-only pointer fields should be measurable and debuggable without
//! following the pointed-to allocation or bounding the target type.

use mem_dbg::*;

#[derive(MemSize, MemDbg)]
#[mem_size(rec)]
struct RawPointerNode {
    value: usize,
    next: *const RawPointerNode,
    prev: *mut RawPointerNode,
}

struct OpaqueMarker;

#[derive(MemSize, MemDbg)]
#[mem_size(rec)]
struct GenericWeakHandleNode<T> {
    parent: Option<std::rc::Weak<T>>,
    sync_parent: Option<std::sync::Weak<T>>,
}

#[test]
fn raw_pointer_fields_are_counted_as_handles() {
    let mut node = RawPointerNode {
        value: 1,
        next: core::ptr::null(),
        prev: core::ptr::null_mut(),
    };
    node.next = core::ptr::addr_of!(node);
    node.prev = core::ptr::addr_of_mut!(node);

    assert!(node.mem_size(SizeFlags::default()) >= core::mem::size_of::<RawPointerNode>());

    let mut output = String::new();
    node.mem_dbg_depth_on(&mut output, 2, DbgFlags::default())
        .unwrap();
    assert!(output.contains("RawPointerNode"));
}

#[test]
fn weak_pointer_fields_do_not_bound_target_type() {
    let node = GenericWeakHandleNode::<OpaqueMarker> {
        parent: Some(std::rc::Weak::new()),
        sync_parent: Some(std::sync::Weak::new()),
    };

    assert!(
        node.mem_size(SizeFlags::default())
            >= core::mem::size_of::<GenericWeakHandleNode<OpaqueMarker>>()
    );

    let mut output = String::new();
    node.mem_dbg_depth_on(&mut output, 1, DbgFlags::default())
        .unwrap();
    assert!(output.contains("GenericWeakHandleNode"));
}
