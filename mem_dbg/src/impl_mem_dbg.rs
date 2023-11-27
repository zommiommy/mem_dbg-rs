/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::marker::PhantomData;

use crate::{Flags, MemDbgImpl};

macro_rules! impl_mem_dbg {
     ($($ty:ty),*) => {$(
 impl MemDbgImpl for $ty {}
     )*};
 }

impl_mem_dbg! {
    (), bool, char, f32, f64,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
}

impl<T: MemDbgImpl> MemDbgImpl for &'_ T {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        depth: usize,
        max_depth: usize,
        is_last: bool,
        flags: Flags,
    ) -> core::fmt::Result {
        if flags.contains(Flags::FOLLOW_REFS) {
            (**self)._mem_dbg_rec_on(writer, depth, max_depth, is_last, flags)
        } else {
            Ok(())
        }
    }
}

impl<T: MemDbgImpl> MemDbgImpl for &'_ mut T {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        depth: usize,
        max_depth: usize,
        is_last: bool,
        flags: Flags,
    ) -> core::fmt::Result {
        if flags.contains(Flags::FOLLOW_REFS) {
            (**self)._mem_dbg_rec_on(writer, depth, max_depth, is_last, flags)
        } else {
            Ok(())
        }
    }
}

impl<T: MemDbgImpl> MemDbgImpl for [T] {}

impl<T: MemDbgImpl> MemDbgImpl for Option<T> {}

impl<T: MemDbgImpl, const N: usize> MemDbgImpl for [T; N] {}

#[cfg(feature = "alloc")]
impl<T: MemDbgImpl> MemDbgImpl for Vec<T> {}

#[cfg(feature = "alloc")]
impl<T: MemDbgImpl> MemDbgImpl for Box<[T]> {}

impl<T: ?Sized> MemDbgImpl for PhantomData<T> {}

#[cfg(feature = "mmap_rs")]
impl MemDbgImpl for mmap_rs::Mmap {}

#[cfg(feature = "mmap_rs")]
impl MemDbgImpl for mmap_rs::MmapMut {}
