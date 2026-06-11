/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Tests that lock guards deduplicate their target through the same map as
//! plain references under `SizeFlags::FOLLOW_REFS`.

#![cfg(feature = "std")]

use anyhow::Result;
use core::mem::size_of;
use mem_dbg::{MemSize, SizeFlags};
use std::sync::{RwLock, RwLockReadGuard};

#[test]
fn test_two_read_guards_count_target_once() -> Result<()> {
    let lock = RwLock::new(vec![0_u8; 100]);
    let g1 = lock.read().unwrap();
    let g2 = lock.read().unwrap();

    let guards = (&g1, &g2);
    let size = guards.mem_size(SizeFlags::FOLLOW_REFS);

    // Two references, two guards, and the guarded vector counted once.
    let expected = 2 * size_of::<&RwLockReadGuard<'_, Vec<u8>>>()
        + 2 * size_of::<RwLockReadGuard<'_, Vec<u8>>>()
        + size_of::<Vec<u8>>()
        + 100;
    assert_eq!(size, expected);
    Ok(())
}

#[test]
fn test_guard_follows_target_like_a_reference() -> Result<()> {
    let lock = RwLock::new(vec![0_u8; 100]);
    let guard = lock.read().unwrap();

    // Without FOLLOW_REFS the guard is a handle only.
    assert_eq!(
        guard.mem_size(SizeFlags::default()),
        size_of::<RwLockReadGuard<'_, Vec<u8>>>()
    );

    // With FOLLOW_REFS the guarded value is counted in full, like `&T`.
    assert_eq!(
        guard.mem_size(SizeFlags::FOLLOW_REFS),
        size_of::<RwLockReadGuard<'_, Vec<u8>>>() + size_of::<Vec<u8>>() + 100
    );
    Ok(())
}
