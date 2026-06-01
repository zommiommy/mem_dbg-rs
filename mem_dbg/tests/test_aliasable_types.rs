#![cfg(all(feature = "aliasable", feature = "std", feature = "derive"))]

//! The `aliasable` wrapper types are memory-layout twins of their
//! `core`/`alloc` counterparts, so their `MemSize`/`MemDbg` behaviour must
//! match `Box`, `Vec`, `String`, and `&mut T` respectively.

use aliasable::AliasableMut;
use aliasable::boxed::AliasableBox;
use aliasable::string::AliasableString;
use aliasable::vec::AliasableVec;
use mem_dbg::*;

#[test]
fn test_aliasable_box_matches_box() {
    let boxed = Box::new(42u64);
    let aliasable = AliasableBox::from_unique(Box::new(42u64));

    // `AliasableBox<T>` has the same layout as `Box<T>` and owns its heap `T`.
    assert_eq!(
        aliasable.mem_size(SizeFlags::default()),
        boxed.mem_size(SizeFlags::default()),
    );
    assert_eq!(
        aliasable.mem_size(SizeFlags::default()),
        core::mem::size_of::<AliasableBox<u64>>() + core::mem::size_of::<u64>(),
    );
}

#[test]
fn test_aliasable_box_follows_heap() {
    let inner = vec![1i32, 2, 3];
    let boxed = Box::new(inner.clone());
    let aliasable = AliasableBox::from_unique(Box::new(inner));

    assert_eq!(
        aliasable.mem_size(SizeFlags::default()),
        boxed.mem_size(SizeFlags::default()),
    );
}

#[test]
fn test_aliasable_vec_flat_payload() {
    let v: AliasableVec<i32> = AliasableVec::from_unique(vec![1, 2, 3, 4]);
    let len = 4;
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        core::mem::size_of::<AliasableVec<i32>>() + len * core::mem::size_of::<i32>(),
    );
    // No public `capacity()` accessor, so `CAPACITY` is a documented no-op.
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default()),
    );
}

#[test]
fn test_aliasable_vec_heap_payload() {
    let v: AliasableVec<String> =
        AliasableVec::from_unique(vec!["alpha".to_string(), "beta".to_string()]);
    let expected = core::mem::size_of::<AliasableVec<String>>()
        + "alpha".to_string().mem_size(SizeFlags::default())
        + "beta".to_string().mem_size(SizeFlags::default());
    assert_eq!(v.mem_size(SizeFlags::default()), expected);
}

#[test]
fn test_aliasable_string() {
    let s = AliasableString::from_unique(String::from("hello"));
    assert_eq!(
        s.mem_size(SizeFlags::default()),
        core::mem::size_of::<AliasableString>() + "hello".len(),
    );
}

#[test]
fn test_aliasable_mut_matches_mut_ref() {
    let mut value = 7u64;
    let aliasable = AliasableMut::from_unique(&mut value);

    // Default: handle only, like `&mut T`.
    assert_eq!(
        aliasable.mem_size(SizeFlags::default()),
        core::mem::size_of::<&mut u64>(),
    );
    // FOLLOW_REFS: handle plus pointee, like `&mut T`.
    assert_eq!(
        aliasable.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<&mut u64>() + core::mem::size_of::<u64>(),
    );
}

#[derive(MemSize, MemDbg)]
struct AliasableFields<'a> {
    boxed: AliasableBox<u64>,
    vec: AliasableVec<i32>,
    string: AliasableString,
    mutable: AliasableMut<'a, u64>,
}

#[test]
fn test_aliasable_fields_derive_mem_dbg() {
    let mut target = 9u64;
    let fields = AliasableFields {
        boxed: AliasableBox::from_unique(Box::new(1)),
        vec: AliasableVec::from_unique(vec![1, 2, 3]),
        string: AliasableString::from_unique(String::from("hi")),
        mutable: AliasableMut::from_unique(&mut target),
    };

    let mut output = String::new();
    fields
        .mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");

    assert!(
        output.contains("AliasableFields"),
        "missing struct name:\n{output}"
    );
    for field in ["boxed", "vec", "string", "mutable"] {
        let needle = format!("╴{field}:");
        assert_eq!(
            output.matches(&needle).count(),
            1,
            "expected one `{needle}` line, got:\n{output}"
        );
    }
}
