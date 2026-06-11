/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Tests for extent-aware deduplication of followed references: two
//! references sharing an address but spanning regions of different sizes
//! must be counted by the largest extent, regardless of encounter order.

use anyhow::Result;
use core::mem::size_of;
use mem_dbg::{MemSize, SizeFlags};

#[test]
fn test_struct_and_first_field_order_independent() -> Result<()> {
    let pair: (u64, u64) = (1, 2);
    let whole: &(u64, u64) = &pair;
    let first: &u64 = &pair.0;

    // Two references plus the pair, counted once by its full extent.
    let expected = 2 * size_of::<&u64>() + size_of::<(u64, u64)>();
    assert_eq!((whole, first).mem_size(SizeFlags::FOLLOW_REFS), expected);
    assert_eq!((first, whole).mem_size(SizeFlags::FOLLOW_REFS), expected);
    Ok(())
}

#[test]
fn test_overlapping_slices_order_independent() -> Result<()> {
    let v = [1_u8, 2, 3, 4];
    let big: &[u8] = &v[..];
    let small: &[u8] = &v[..2];

    // Two fat references plus the slice, counted once by its full extent.
    let expected = 2 * size_of::<&[u8]>() + size_of::<[u8; 4]>();
    assert_eq!((big, small).mem_size(SizeFlags::FOLLOW_REFS), expected);
    assert_eq!((small, big).mem_size(SizeFlags::FOLLOW_REFS), expected);
    Ok(())
}

#[test]
fn test_equal_extents_still_deduplicated() -> Result<()> {
    let x = 42_u64;
    let r1 = &x;
    let r2 = &x;

    let expected = 2 * size_of::<&u64>() + size_of::<u64>();
    assert_eq!((r1, r2).mem_size(SizeFlags::FOLLOW_REFS), expected);
    Ok(())
}
