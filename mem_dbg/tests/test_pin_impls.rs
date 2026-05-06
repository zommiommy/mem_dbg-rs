#![cfg(all(feature = "std", feature = "derive"))]

//! `Pin<P>` should keep the same memory accounting and debug traversal policy
//! as `P`, including `&T`/`Rc<T>`/`Arc<T>` deduplication.

use mem_dbg::*;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

#[derive(MemSize, MemDbg)]
struct PinFields<'a> {
    boxed: Pin<Box<usize>>,
    borrowed: Pin<&'a usize>,
    mutable: Pin<&'a mut usize>,
    rc: Pin<Rc<usize>>,
    arc: Pin<Arc<usize>>,
}

#[test]
fn pin_size_matches_wrapped_pointer_policy() {
    let plain_box = Box::new(1usize);
    let pinned_box = Box::pin(1usize);
    assert_eq!(
        pinned_box.mem_size(SizeFlags::default()),
        plain_box.mem_size(SizeFlags::default())
    );

    let value = 2usize;
    let plain_ref: &usize = &value;
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
fn pin_rc_dedup_matches_plain_rc() {
    // Two `Pin<Rc<T>>` aliasing the same allocation must dedup under
    // `FOLLOW_RCS` exactly as two plain `Rc<T>` would, and must dedup with
    // a plain `Rc<T>` to the same allocation.
    #[derive(MemSize, MemDbg)]
    struct Pinned {
        a: Pin<Rc<[u8; 1024]>>,
        b: Pin<Rc<[u8; 1024]>>,
    }

    #[derive(MemSize, MemDbg)]
    struct Plain {
        a: Rc<[u8; 1024]>,
        b: Rc<[u8; 1024]>,
    }

    #[derive(MemSize, MemDbg)]
    struct Mixed {
        pinned: Pin<Rc<[u8; 1024]>>,
        plain: Rc<[u8; 1024]>,
    }

    let shared = Rc::new([0u8; 1024]);

    let pinned = Pinned {
        a: Pin::new(Rc::clone(&shared)),
        b: Pin::new(Rc::clone(&shared)),
    };
    let plain = Plain {
        a: Rc::clone(&shared),
        b: Rc::clone(&shared),
    };
    let mixed = Mixed {
        pinned: Pin::new(Rc::clone(&shared)),
        plain: Rc::clone(&shared),
    };

    // mem_size: payload counted once.
    let plain_size = plain.mem_size(SizeFlags::FOLLOW_RCS);
    assert_eq!(pinned.mem_size(SizeFlags::FOLLOW_RCS), plain_size);
    assert_eq!(mixed.mem_size(SizeFlags::FOLLOW_RCS), plain_size);

    // mem_dbg: one first-encounter `@`, one back-reference `→`, regardless of
    // whether the alias goes through `Pin<Rc>` or plain `Rc`.
    for (label, output) in [
        ("plain", dbg_with_follow_rcs(&plain)),
        ("pinned", dbg_with_follow_rcs(&pinned)),
        ("mixed", dbg_with_follow_rcs(&mixed)),
    ] {
        assert_eq!(
            output.matches("@ 0x").count(),
            1,
            "{label}: expected one first-encounter marker, got:\n{output}"
        );
        assert_eq!(
            output.matches("→ 0x").count(),
            1,
            "{label}: expected one back-reference marker, got:\n{output}"
        );
    }
}

fn dbg_with_follow_rcs<T: MemDbg>(value: &T) -> String {
    let mut output = String::new();
    value
        .mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .expect("mem_dbg_on failed");
    output
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

    // Parent struct name appears once.
    assert!(
        output.contains("PinFields"),
        "missing struct name:\n{output}"
    );

    // Each field name appears as a tree-glyph-prefixed line, exactly once.
    // Using `╴<field>:` avoids matching identifiers inside fully-qualified
    // type names (which use `::`, not `╴`).
    for field in ["boxed", "borrowed", "mutable", "rc", "arc"] {
        let needle = format!("╴{field}:");
        assert_eq!(
            output.matches(&needle).count(),
            1,
            "expected one `{needle}` line, got:\n{output}"
        );
    }
}
