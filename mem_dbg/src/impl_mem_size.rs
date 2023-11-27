/*
 * SPDX-FileCopyrightText: 2023 Inria
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

/// Note that references are not considered to be copy types
/// because in case [`SizeFlags::FOLLOW_REFS`] is set we need
/// to follow the reference. This implies that the optimization
/// for copy types cannot be applied to references.
///
/// TODO
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

impl<T: CopyType + MemSize, const N: usize> CopyType for [T; N] {
    type Copy = T::Copy;
}

impl<T: MemSize, const N: usize> MemSize for [T; N] {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
            + self
                .iter()
                .map(|x| x.mem_size(flags) - core::mem::size_of::<T>())
                .sum::<usize>()
    }
}

impl<T> CopyType for Vec<T> {
    type Copy = False;
}

pub trait MemSizeHelper<T: Boolean> {
    fn _mem_size(&self, flags: SizeFlags) -> usize;
}

impl<T: CopyType> MemSize for Vec<T>
where
    Vec<T>: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <Vec<T> as MemSizeHelper<<T as CopyType>::Copy>>::_mem_size(&self, flags)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<True> for Vec<T> {
    #[inline(always)]
    fn _mem_size(&self, flags: SizeFlags) -> usize {
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
    fn _mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>()
                + self.iter().map(|x| x.mem_size(flags)).sum::<usize>()
                + (self.capacity() - self.len()) * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>() + self.iter().map(|x| x.mem_size(flags)).sum::<usize>()
        }
    }
}

impl<T: CopyType> MemSize for [T]
where
    [T]: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <[T] as MemSizeHelper<<T as CopyType>::Copy>>::_mem_size(&self, flags)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<True> for [T] {
    #[inline(always)]
    fn _mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<usize>() + self.len() * core::mem::size_of::<T>()
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
impl<T: CopyType + MemSize> MemSizeHelper<False> for [T] {
    #[inline(always)]
    fn _mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<usize>() + self.iter().map(|x| x.mem_size(flags)).sum::<usize>()
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
