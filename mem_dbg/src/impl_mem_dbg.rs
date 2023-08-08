/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::MemDbg;

macro_rules! impl_mem_dbg {
     ($($ty:ty),*) => {$(
 impl MemDbg for $ty {}
     )*};
 }

impl_mem_dbg! {
    (), bool, char, f32, f64,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    &'_ str
}

impl<T: MemDbg> MemDbg for &'_ [T] {}

impl<T: MemDbg> MemDbg for Option<T> {}

#[cfg(feature = "alloc")]
impl<T: MemDbg> MemDbg for Vec<T> {}

#[cfg(feature = "alloc")]
impl<T: MemDbg> MemDbg for Box<[T]> {}

#[cfg(feature = "mmap_rs")]
impl MemDbg for mmap_rs::Mmap {}

#[cfg(feature = "mmap_rs")]
impl MemDbg for mmap_rs::MmapMut {}
