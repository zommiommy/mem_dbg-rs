/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Tests for `Rc`/`Arc` with unsized pointees (`str`, slices).

#![cfg(feature = "std")]

use anyhow::Result;
use core::mem::{align_of, size_of};
use mem_dbg::{DbgFlags, MemDbg, MemSize, SizeFlags};
use std::rc::Rc;
use std::sync::Arc;

/// Size of the `RcInner`/`ArcInner` allocation: two-word header extended
/// with the payload and padded to the allocation alignment, as computed by
/// the standard library.
fn shared_alloc_size(payload_size: usize, payload_align: usize) -> usize {
    let align = align_of::<usize>().max(payload_align);
    let header = (2 * size_of::<usize>()).next_multiple_of(payload_align);
    (header + payload_size).next_multiple_of(align)
}

#[test]
fn test_rc_str() -> Result<()> {
    let rc: Rc<str> = Rc::from("hello");

    // Without FOLLOW_RCS the handle is a fat pointer.
    assert_eq!(rc.mem_size(SizeFlags::default()), size_of::<Rc<str>>());

    // With FOLLOW_RCS the whole allocation is counted.
    assert_eq!(
        rc.mem_size(SizeFlags::FOLLOW_RCS),
        size_of::<Rc<str>>() + shared_alloc_size(5, 1)
    );
    Ok(())
}

#[test]
fn test_arc_slice() -> Result<()> {
    let arc: Arc<[u32]> = Arc::from(vec![1_u32, 2, 3, 4, 5]);

    assert_eq!(arc.mem_size(SizeFlags::default()), size_of::<Arc<[u32]>>());
    assert_eq!(
        arc.mem_size(SizeFlags::FOLLOW_RCS),
        size_of::<Arc<[u32]>>() + shared_alloc_size(5 * size_of::<u32>(), align_of::<u32>())
    );
    Ok(())
}

#[test]
fn test_rc_str_clones_deduplicated() -> Result<()> {
    let rc: Rc<str> = Rc::from("hello");
    let pair = (rc.clone(), rc);

    assert_eq!(
        pair.mem_size(SizeFlags::FOLLOW_RCS),
        2 * size_of::<Rc<str>>() + shared_alloc_size(5, 1)
    );
    Ok(())
}

#[test]
fn test_arc_nonflat_slice_recurses() -> Result<()> {
    let arc: Arc<[String]> = Arc::from(vec!["ab".to_owned(), "cdef".to_owned()]);

    // The allocation plus the heap content of the strings.
    assert_eq!(
        arc.mem_size(SizeFlags::FOLLOW_RCS),
        size_of::<Arc<[String]>>()
            + shared_alloc_size(2 * size_of::<String>(), align_of::<String>())
            + 2
            + 4
    );
    Ok(())
}

#[test]
fn test_rc_str_mem_dbg() -> Result<()> {
    let rc: Rc<str> = Rc::from("hello");
    let mut output = String::new();
    rc.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_RCS)?;
    assert!(output.contains("@ 0x"), "missing address marker: {output}");
    Ok(())
}
