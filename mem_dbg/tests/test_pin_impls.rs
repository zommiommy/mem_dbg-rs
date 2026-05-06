#![cfg(all(feature = "std", feature = "derive"))]

//! `Pin<P>` should keep the same memory accounting and debug traversal policy
//! as `P`.

use mem_dbg::*;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

#[derive(MemSize, MemDbg)]
#[mem_size(rec)]
struct PinFields<'a> {
    boxed: Pin<Box<usize>>,
    borrowed: Pin<&'a usize>,
    mutable: Pin<&'a mut usize>,
    rc: Pin<Rc<usize>>,
    arc: Pin<Arc<usize>>,
}

#[test]
fn pin_size_matches_wrapped_pointer_policy() {
    let boxed = Box::pin(1usize);
    assert_eq!(
        boxed.mem_size(SizeFlags::default()),
        Box::new(1usize).mem_size(SizeFlags::default())
    );

    let value = 2usize;
    let plain_ref = &value;
    let pinned_ref = Pin::new(&value);
    assert_eq!(
        pinned_ref.mem_size(SizeFlags::default()),
        <&usize as MemSize>::mem_size(&plain_ref, SizeFlags::default())
    );
    assert_eq!(
        pinned_ref.mem_size(SizeFlags::FOLLOW_REFS),
        <&usize as MemSize>::mem_size(&plain_ref, SizeFlags::FOLLOW_REFS)
    );

    let mut mutable_value = 5usize;
    let pinned_mut = Pin::new(&mut mutable_value);
    assert_eq!(
        pinned_mut.mem_size(SizeFlags::default()),
        core::mem::size_of::<&mut usize>()
    );
    assert_eq!(
        pinned_mut.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<&mut usize>() + core::mem::size_of::<usize>()
    );

    let rc = Rc::new(3usize);
    let pinned_rc = Pin::new(Rc::clone(&rc));
    assert_eq!(
        pinned_rc.mem_size(SizeFlags::default()),
        Rc::clone(&rc).mem_size(SizeFlags::default())
    );
    assert_eq!(
        pinned_rc.mem_size(SizeFlags::FOLLOW_RCS),
        Rc::clone(&rc).mem_size(SizeFlags::FOLLOW_RCS)
    );

    let arc = Arc::new(4usize);
    let pinned_arc = Pin::new(Arc::clone(&arc));
    assert_eq!(
        pinned_arc.mem_size(SizeFlags::default()),
        Arc::clone(&arc).mem_size(SizeFlags::default())
    );
    assert_eq!(
        pinned_arc.mem_size(SizeFlags::FOLLOW_RCS),
        Arc::clone(&arc).mem_size(SizeFlags::FOLLOW_RCS)
    );
}

#[test]
fn pin_fields_derive_mem_dbg() {
    let value = 2usize;
    let mut mutable_value = 5usize;
    let fields = PinFields {
        boxed: Box::pin(1),
        borrowed: Pin::new(&value),
        mutable: Pin::new(&mut mutable_value),
        rc: Pin::new(Rc::new(3)),
        arc: Pin::new(Arc::new(4)),
    };

    let mut output = String::new();
    fields
        .mem_dbg_depth_on(
            &mut output,
            2,
            DbgFlags::default() | DbgFlags::FOLLOW_REFS | DbgFlags::FOLLOW_RCS,
        )
        .unwrap();

    assert!(output.contains("PinFields"));
    assert!(output.contains("boxed"));
    assert!(output.contains("borrowed"));
    assert!(output.contains("mutable"));
    assert!(output.contains("rc"));
    assert!(output.contains("arc"));
}
