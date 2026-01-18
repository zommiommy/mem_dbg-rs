#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::rc::Rc;
use std::sync::Arc;

#[test]
fn test_box_in_struct() {
    #[derive(MemSize)]
    struct Test {
        boxed: Box<u64>,
    }

    let s = Test {
        boxed: Box::new(42),
    };
    let size = s.mem_size(SizeFlags::default());

    // Box adds its pointer size + the contained value
    let expected = std::mem::size_of::<Box<u64>>() + std::mem::size_of::<u64>();
    assert_eq!(size, expected);
}

#[test]
fn test_rc_deduplication() {
    #[derive(MemSize)]
    struct Test {
        rc1: Rc<[u8; 1000]>,
        rc2: Rc<[u8; 1000]>,
    }

    let shared = Rc::new([0u8; 1000]);
    let s = Test {
        rc1: Rc::clone(&shared),
        rc2: Rc::clone(&shared),
    };

    // Without FOLLOW_RC: just the two Rc pointer sizes
    let size_no_follow = s.mem_size(SizeFlags::default());
    assert_eq!(size_no_follow, 2 * std::mem::size_of::<Rc<[u8; 1000]>>());

    // With FOLLOW_RC: the shared data should only be counted once
    let size_with_follow = s.mem_size(SizeFlags::FOLLOW_RC);

    // Two Rc pointers + one RcInner (which contains strong/weak counts + data)
    // The data (1000 bytes) should only appear once, not twice
    let rc_ptr_size = std::mem::size_of::<Rc<[u8; 1000]>>();

    // If deduplication works, size should be much less than if counted twice
    // (2 * rc_ptr_size + 2 * 1000 would be wrong)
    assert!(
        size_with_follow < 2 * rc_ptr_size + 2 * 1000,
        "Deduplication failed: size {} should be less than {}",
        size_with_follow,
        2 * rc_ptr_size + 2 * 1000
    );
}

#[test]
fn test_arc_deduplication() {
    #[derive(MemSize)]
    struct Test {
        arc1: Arc<[u8; 1000]>,
        arc2: Arc<[u8; 1000]>,
    }

    let shared = Arc::new([0u8; 1000]);
    let s = Test {
        arc1: Arc::clone(&shared),
        arc2: Arc::clone(&shared),
    };

    // Without FOLLOW_RC: just the two Arc pointer sizes
    let size_no_follow = s.mem_size(SizeFlags::default());
    assert_eq!(size_no_follow, 2 * std::mem::size_of::<Arc<[u8; 1000]>>());

    // With FOLLOW_RC: the shared data should only be counted once
    let size_with_follow = s.mem_size(SizeFlags::FOLLOW_RC);

    let arc_ptr_size = std::mem::size_of::<Arc<[u8; 1000]>>();

    // If deduplication works, size should be much less than if counted twice
    assert!(
        size_with_follow < 2 * arc_ptr_size + 2 * 1000,
        "Deduplication failed: size {} should be less than {}",
        size_with_follow,
        2 * arc_ptr_size + 2 * 1000
    );
}

#[test]
fn test_reference_deduplication() {
    #[derive(MemSize)]
    struct Test<'a> {
        ref1: &'a [u8; 1000],
        ref2: &'a [u8; 1000],
    }

    let data = [0u8; 1000];
    let s = Test {
        ref1: &data,
        ref2: &data,
    };

    // Without FOLLOW_REFS: just the two reference sizes
    let size_no_follow = s.mem_size(SizeFlags::default());
    assert_eq!(size_no_follow, 2 * std::mem::size_of::<&[u8; 1000]>());

    // With FOLLOW_REFS: the shared data should only be counted once
    let size_with_follow = s.mem_size(SizeFlags::FOLLOW_REFS);

    let ref_size = std::mem::size_of::<&[u8; 1000]>();

    // If deduplication works, size should be much less than if counted twice
    // Expected: 2 * ref_size + 1000 (data counted once)
    // Wrong:    2 * ref_size + 2000 (data counted twice)
    assert!(
        size_with_follow < 2 * ref_size + 2 * 1000,
        "Deduplication failed: size {} should be less than {}",
        size_with_follow,
        2 * ref_size + 2 * 1000
    );

    // More precisely, should be exactly 2 refs + 1000 bytes of data
    assert_eq!(size_with_follow, 2 * ref_size + 1000);
}
