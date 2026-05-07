#![cfg(feature = "std")]

//! Size assertions for `Cow<'_, B>`.
//!
//! Borrowed values use reference-following rules, while owned values use the
//! memory accounting of `B::Owned`.

use mem_dbg::{DbgFlags, MemDbg, MemSize, SizeFlags};
use std::borrow::Cow;

#[test]
fn test_cow_borrowed_str_default() {
    let cow: Cow<'_, str> = Cow::Borrowed("borrowed");

    assert_eq!(
        cow.mem_size(SizeFlags::default()),
        core::mem::size_of::<Cow<'_, str>>()
    );
}

#[test]
fn test_cow_borrowed_str_follow_refs() {
    let value = "borrowed";
    let cow: Cow<'_, str> = Cow::Borrowed(value);

    assert_eq!(
        cow.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<Cow<'_, str>>() + value.len()
    );
}

#[test]
fn test_cow_borrowed_refs_are_deduplicated() {
    let value = "shared";
    let pair = (
        Cow::<'_, str>::Borrowed(value),
        Cow::<'_, str>::Borrowed(value),
    );

    assert_eq!(
        pair.mem_size(SizeFlags::default()),
        core::mem::size_of::<(Cow<'_, str>, Cow<'_, str>)>()
    );
    assert_eq!(
        pair.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<(Cow<'_, str>, Cow<'_, str>)>() + value.len()
    );
}

#[test]
fn test_cow_borrowed_slice_follow_refs() {
    let value = [1_u8, 2, 3, 4];
    let cow: Cow<'_, [u8]> = Cow::Borrowed(&value);

    assert_eq!(
        cow.mem_size(SizeFlags::default()),
        core::mem::size_of::<Cow<'_, [u8]>>()
    );
    assert_eq!(
        cow.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<Cow<'_, [u8]>>() + value.len() * core::mem::size_of::<u8>()
    );
}

#[test]
fn test_cow_owned_str_size_and_capacity() {
    let mut value = String::with_capacity(32);
    value.push_str("owned");
    let len = value.len();
    let capacity = value.capacity();
    let cow: Cow<'_, str> = Cow::Owned(value);

    assert_eq!(
        cow.mem_size(SizeFlags::default()),
        core::mem::size_of::<Cow<'_, str>>() + len
    );
    assert_eq!(
        cow.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<Cow<'_, str>>() + capacity
    );
}

#[test]
fn test_cow_owned_slice_size_and_capacity() {
    let mut value = Vec::with_capacity(8);
    value.extend_from_slice(&[1_u32, 2, 3]);
    let len = value.len();
    let capacity = value.capacity();
    let cow: Cow<'_, [u32]> = Cow::Owned(value);

    assert_eq!(
        cow.mem_size(SizeFlags::default()),
        core::mem::size_of::<Cow<'_, [u32]>>() + len * core::mem::size_of::<u32>()
    );
    assert_eq!(
        cow.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<Cow<'_, [u32]>>() + capacity * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_cow_mem_dbg_borrowed_and_owned() {
    let borrowed: Cow<'_, str> = Cow::Borrowed("borrowed");
    let owned: Cow<'_, str> = Cow::Owned(String::from("owned"));
    let mut output = String::new();

    assert!(
        borrowed
            .mem_dbg_depth_on(&mut output, 1, DbgFlags::FOLLOW_REFS)
            .is_ok()
    );
    output.clear();
    assert!(
        owned
            .mem_dbg_depth_on(&mut output, 1, DbgFlags::default())
            .is_ok()
    );
}
