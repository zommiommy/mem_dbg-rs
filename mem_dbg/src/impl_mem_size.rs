/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::marker::PhantomData;
use core::sync::atomic::*;

use crate::{Boolean, CopyType, False, MemSize, SizeFlags, True};

macro_rules! impl_memory_size {
    ($($ty:ty),*) => {$(
        impl CopyType for $ty {
            type Copy = True;
        }
        impl MemSize for $ty {
            #[inline(always)]
            fn mem_size(&self, _flags: SizeFlags) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    )*};
}

impl_memory_size! {
   (), bool, char, f32, f64,
   u8, u16, u32, u64, u128, usize,
   i8, i16, i32, i64, i128, isize,
   AtomicBool,
   AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
   AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize
}

impl<T: ?Sized + MemSize> CopyType for &'_ T {
    type Copy = False;
}

impl<T: ?Sized + MemSize> MemSize for &'_ T {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            core::mem::size_of::<Self>() + (**self).mem_size(flags)
        } else {
            core::mem::size_of::<Self>()
        }
    }
}

impl<T: ?Sized + MemSize> CopyType for &'_ mut T {
    type Copy = False;
}

impl<T: ?Sized + MemSize> MemSize for &'_ mut T {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <&'_ T as MemSize>::mem_size(&&**self, flags)
    }
}

impl<T: CopyType + MemSize> CopyType for Option<T> {
    type Copy = T::Copy;
}

impl<T: MemSize> MemSize for Option<T> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
            + self
                .as_ref()
                .map_or(0, |x| x.mem_size(flags) - core::mem::size_of::<T>())
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
impl<T: ?Sized + MemSize> MemSize for Box<T> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.as_ref().mem_size(flags)
    }
}

impl<T> CopyType for PhantomData<T> {
    type Copy = True;
}

impl<T: ?Sized> MemSize for PhantomData<T> {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        0
    }
}

impl CopyType for str {
    type Copy = False;
}

impl MemSize for str {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<usize>() + self.len()
    }
}

impl CopyType for String {
    type Copy = False;
}

impl MemSize for String {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>() + self.capacity()
        } else {
            core::mem::size_of::<Self>() + self.len()
        }
    }
}

/// A helper trait that makes it possible to implement differently
/// the size computation for arrays, vectors, and slices of
/// [`Copy`] types.
///
/// See [`crate::CopyType`] for more information.
pub trait MemSizeHelper<T: Boolean> {
    fn mem_size_impl(&self, flags: SizeFlags) -> usize;
}

// Arrays

impl<T: CopyType + MemSize, const N: usize> CopyType for [T; N] {
    type Copy = T::Copy;
}

impl<T: CopyType, const N: usize> MemSize for [T; N]
where
    [T; N]: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <[T; N] as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags)
    }
}

impl<T: MemSize, const N: usize> MemSizeHelper<True> for [T; N] {
    #[inline(always)]
    fn mem_size_impl(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.len() * core::mem::size_of::<T>()
    }
}

impl<T: MemSize, const N: usize> MemSizeHelper<False> for [T; N] {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
            + self
                .iter()
                .map(|x| x.mem_size(flags) - core::mem::size_of::<T>())
                .sum::<usize>()
    }
}

// Vectors

impl<T> CopyType for Vec<T> {
    type Copy = False;
}

impl<T: CopyType> MemSize for Vec<T>
where
    Vec<T>: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <Vec<T> as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<True> for Vec<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>() + self.capacity() * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>() + self.len() * core::mem::size_of::<T>()
        }
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<False> for Vec<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>()
                + self.iter().map(|x| x.mem_size(flags)).sum::<usize>()
                + (self.capacity() - self.len()) * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>() + self.iter().map(|x| x.mem_size(flags)).sum::<usize>()
        }
    }
}

// Slices

impl<T: CopyType> MemSize for [T]
where
    [T]: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <[T] as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<True> for [T] {
    #[inline(always)]
    fn mem_size_impl(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<usize>() + std::mem::size_of_val(self)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<False> for [T] {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<usize>() + self.iter().map(|x| x.mem_size(flags)).sum::<usize>()
    }
}

macro_rules! impl_mem_size_tuples {
    ($(($idx:tt => $ty:ident),)*) => {
        impl<$($ty,)*> CopyType for ($($ty,)*)  {
            type Copy = False;
		}

		impl<$($ty: MemSize,)*> MemSize for ($($ty,)*)
        {
            #[inline(always)]
            fn mem_size(&self, flags: SizeFlags) -> usize {
                0
                $(
                    + self.$idx.mem_size(flags)
                )*
            }
        }
    }
}

macro_rules! impl_tuples_muncher {
    (($idx:tt => $ty:ident), $(($i:tt => $t:ident),)*) => {
        impl_mem_size_tuples!(($idx => $ty), $(($i => $t),)*);
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

#[cfg(feature = "mmap_rs")]
impl MemSize for mmap_rs::Mmap {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(feature = "mmap_rs")]
impl MemSize for mmap_rs::MmapMut {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}
