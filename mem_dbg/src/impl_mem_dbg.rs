/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::{marker::PhantomData, sync::atomic::*};

use crate::{impl_mem_size::MemSizeHelper, CopyType, DbgFlags, MemDbgImpl};

macro_rules! impl_mem_dbg {
     ($($ty:ty),*) => {$(
 impl MemDbgImpl for $ty {}
     )*};
 }

impl_mem_dbg! {
    (), bool, char, f32, f64,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    AtomicBool,
    AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
    AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize
}

impl<T: ?Sized + MemDbgImpl> MemDbgImpl for &'_ T {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        depth: usize,
        max_depth: usize,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            (**self)._mem_dbg_rec_on(writer, total_size, depth, max_depth, is_last, flags)
        } else {
            Ok(())
        }
    }
}

impl<T: ?Sized + MemDbgImpl> MemDbgImpl for &'_ mut T {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        depth: usize,
        max_depth: usize,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            (**self)._mem_dbg_rec_on(writer, total_size, depth, max_depth, is_last, flags)
        } else {
            Ok(())
        }
    }
}

impl<T: CopyType + MemDbgImpl> MemDbgImpl for [T] where [T]: MemSizeHelper<<T as CopyType>::Copy> {}

impl<T: MemDbgImpl> MemDbgImpl for Option<T> {}

impl<T: CopyType + MemDbgImpl, const N: usize> MemDbgImpl for [T; N] where
    [T; N]: MemSizeHelper<<T as CopyType>::Copy>
{
}

#[cfg(feature = "alloc")]
impl<T: CopyType + MemDbgImpl> MemDbgImpl for Vec<T> where
    Vec<T>: MemSizeHelper<<T as CopyType>::Copy>
{
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + MemDbgImpl> MemDbgImpl for Box<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        depth: usize,
        max_depth: usize,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.as_ref()
            ._mem_dbg_rec_on(writer, total_size, depth, max_depth, is_last, flags)
    }
}

impl<T: ?Sized> MemDbgImpl for PhantomData<T> {}

impl MemDbgImpl for str {}

impl MemDbgImpl for String {}

#[cfg(feature = "mmap_rs")]
impl MemDbgImpl for mmap_rs::Mmap {}

#[cfg(feature = "mmap_rs")]
impl MemDbgImpl for mmap_rs::MmapMut {}

macro_rules! impl_mem_dbg_tuples {
    ($(($idx:tt => $ty:ident),)*) => {
        impl<$($ty: crate::MemSize + MemDbgImpl,)*> MemDbgImpl for ($($ty,)*)  {
            fn _mem_dbg_rec_on(
                &self,
                writer: &mut impl core::fmt::Write,
                total_size: usize,
                depth: usize,
                max_depth: usize,
                is_last: bool,
                flags: DbgFlags,
            ) -> core::fmt::Result {
                Ok(())
            }
        }
    }
}

macro_rules! impl_tuples_muncher {
    (($idx:tt => $ty:ident), $(($i:tt => $t:ident),)*) => {
        impl_mem_dbg_tuples!(($idx => $ty), $(($i => $t),)*);
        impl_tuples_muncher!($(($i => $t),)*);
    };
    (($idx:tt => $ty:ident)) => {
        impl_mem_size_tuples!(($idx => $ty));
    };
    () => {};
}

impl_tuples_muncher!(
    (9 => T9),
    (8 => T8),
    (7 => T7),
    (6 => T6),
    (5 => T5),
    (4 => T4),
    (3 => T3),
    (2 => T2),
    (1 => T1),
    (0 => T0),
);
