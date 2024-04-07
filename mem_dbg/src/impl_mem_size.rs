/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::marker::{PhantomData, PhantomPinned};
use core::num::*;
use core::ops::Deref;
use core::sync::atomic::*;
use std::collections::{HashMap, HashSet};

use crate::{Boolean, CopyType, False, MemSize, SizeFlags, True};

// Primitive types, atomic types, ()

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
   AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
   NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
   NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
   PhantomPinned
}

// Strings

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

// PhantomData

impl<T> CopyType for PhantomData<T> {
    type Copy = True;
}

impl<T: ?Sized> MemSize for PhantomData<T> {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        0
    }
}

// References: we recurse only if FOLLOW_REFS is set

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

// Option

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

// Box

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
impl<T: ?Sized + MemSize> MemSize for Box<T> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.as_ref().mem_size(flags)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(feature = "alloc")]
impl<T: MemSize> MemSize for Arc<T> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>() + self.as_ref().mem_size(flags)
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
        core::mem::size_of::<Self>()
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

// Tuples

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

impl<R> CopyType for fn() -> R {
    type Copy = True;
}
impl<R> MemSize for fn() -> R {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}
impl<A, R> CopyType for fn(A) -> R {
    type Copy = True;
}
impl<A, R> MemSize for fn(A) -> R {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}
impl<A, B, R> CopyType for fn(A, B) -> R {
    type Copy = True;
}
impl<A, B, R> MemSize for fn(A, B) -> R {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}
impl<A, B, C, R> CopyType for fn(A, B, C) -> R {
    type Copy = True;
}
impl<A, B, C, R> MemSize for fn(A, B, C) -> R {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}
impl<A, B, C, D, R> CopyType for fn(A, B, C, D) -> R {
    type Copy = True;
}
impl<A, B, C, D, R> MemSize for fn(A, B, C, D) -> R {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::Range<Idx> {
    type Copy = True;
}
impl<Idx: MemSize> MemSize for core::ops::Range<Idx> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.start.mem_size(flags) + self.end.mem_size(flags)
            - 2 * core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeFrom<Idx> {
    type Copy = True;
}
impl<Idx: MemSize> MemSize for core::ops::RangeFrom<Idx> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.start.mem_size(flags) - core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeInclusive<Idx> {
    type Copy = True;
}
impl<Idx: MemSize> MemSize for core::ops::RangeInclusive<Idx> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.start().mem_size(flags) + self.end().mem_size(flags)
            - 2 * core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeTo<Idx> {
    type Copy = True;
}
impl<Idx: MemSize> MemSize for core::ops::RangeTo<Idx> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.end.mem_size(flags) - core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeToInclusive<Idx> {
    type Copy = True;
}
impl<Idx: MemSize> MemSize for core::ops::RangeToInclusive<Idx> {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() + self.end.mem_size(flags) - core::mem::size_of::<Idx>()
    }
}

#[cfg(feature = "rand")]
impl CopyType for rand::rngs::SmallRng {
    type Copy = True;
}
#[cfg(feature = "rand")]
impl MemSize for rand::rngs::SmallRng {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl<T: CopyType> CopyType for core::cell::RefCell<T> {
    type Copy = T::Copy;
}
impl<T: MemSize> MemSize for core::cell::RefCell<T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>() + self.borrow().mem_size(flags)
    }
}

impl<T: CopyType> CopyType for core::cell::Cell<T> {
    type Copy = T::Copy;
}
impl<T: MemSize> MemSize for core::cell::Cell<T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + unsafe { (*self.as_ptr()).mem_size(flags) }
    }
}

impl<T: CopyType> CopyType for core::cell::OnceCell<T> {
    type Copy = T::Copy;
}
impl<T: MemSize> MemSize for core::cell::OnceCell<T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>() + self.get().mem_size(flags)
    }
}

impl<T: CopyType> CopyType for core::cell::UnsafeCell<T> {
    type Copy = T::Copy;
}
impl<T: MemSize> MemSize for core::cell::UnsafeCell<T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + unsafe { (*self.get()).mem_size(flags) }
    }
}

#[cfg(feature = "std")]
impl<T: CopyType> CopyType for std::sync::Mutex<T> {
    type Copy = False;
}
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::Mutex<T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + self.lock().unwrap().mem_size(flags)
    }
}

#[cfg(feature = "std")]
impl<T: CopyType> CopyType for std::sync::RwLock<T> {
    type Copy = False;
}
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLock<T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + self.read().unwrap().mem_size(flags)
    }
}

#[cfg(feature = "std")]
impl<T: CopyType> CopyType for std::sync::MutexGuard<'_, T> {
    type Copy = False;
}
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::MutexGuard<'_, T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            core::mem::size_of::<Self>() - core::mem::size_of::<T>() + self.deref().mem_size(flags)
        } else {
            0
        }
    }
}

#[cfg(feature = "std")]
impl<T: CopyType> CopyType for std::sync::RwLockReadGuard<'_, T> {
    type Copy = False;
}
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLockReadGuard<'_, T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            core::mem::size_of::<Self>() - core::mem::size_of::<T>() + self.deref().mem_size(flags)
        } else {
            0
        }
    }
}

#[cfg(feature = "std")]
impl<T: CopyType> CopyType for std::sync::RwLockWriteGuard<'_, T> {
    type Copy = False;
}
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLockWriteGuard<'_, T> {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            core::mem::size_of::<Self>() - core::mem::size_of::<T>() + self.deref().mem_size(flags)
        } else {
            0
        }
    }
}

#[cfg(feature = "std")]
impl CopyType for std::path::Path {
    type Copy = False;
}
#[cfg(feature = "std")]
impl MemSize for std::path::Path {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        self.as_os_str().mem_size(flags)
    }
}

#[cfg(feature = "std")]
impl CopyType for std::path::PathBuf {
    type Copy = False;
}
#[cfg(feature = "std")]
impl MemSize for std::path::PathBuf {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>() + self.capacity().mem_size(flags)
        } else {
            self.as_os_str().mem_size(flags)
        }
    }
}

#[cfg(feature = "std")]
impl CopyType for std::ffi::OsStr {
    type Copy = False;
}
#[cfg(feature = "std")]
impl MemSize for std::ffi::OsStr {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            self.as_encoded_bytes().len()
        } else {
            0
        }
    }
}

#[cfg(feature = "std")]
impl CopyType for std::ffi::OsString {
    type Copy = False;
}
#[cfg(feature = "std")]
impl MemSize for std::ffi::OsString {
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::CAPACITY) {
                self.capacity().mem_size(flags)
            } else {
                self.len().mem_size(flags)
            }
    }
}

#[cfg(feature = "mmap_rs")]
impl CopyType for mmap_rs::Mmap {
    type Copy = False;
}
#[cfg(feature = "mmap_rs")]
impl MemSize for mmap_rs::Mmap {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(feature = "mmap_rs")]
impl CopyType for mmap_rs::MmapMut {
    type Copy = False;
}
#[cfg(feature = "mmap_rs")]
impl MemSize for mmap_rs::MmapMut {
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Hash-based containers from the standard library
//
// If the standard library changes load factor, this code will have to change
// accordingly.

// Straight from hashbrown
fn capacity_to_buckets(cap: usize) -> Option<usize> {
    debug_assert_ne!(cap, 0);

    // For small tables we require at least 1 empty bucket so that lookups are
    // guaranteed to terminate if an element doesn't exist in the table.
    if cap < 8 {
        // We don't bother with a table size of 2 buckets since that can only
        // hold a single element. Instead we skip directly to a 4 bucket table
        // which can hold 3 elements.
        return Some(if cap < 4 { 4 } else { 8 });
    }

    // Otherwise require 1/8 buckets to be empty (87.5% load)
    //
    // Be careful when modifying this, calculate_layout relies on the
    // overflow check here.
    let adjusted_cap = cap.checked_mul(8)? / 7;

    // Any overflows will have been caught by the checked_mul. Also, any
    // rounding errors from the division above will be cleaned up by
    // next_power_of_two (which can't overflow because of the previous division).
    Some(adjusted_cap.next_power_of_two())
}

impl<T: CopyType> MemSize for HashSet<T>
where
    HashSet<T>: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <HashSet<T> as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags)
    }
}

// Add to the given size the space occupied on the stack by the hash set, by the unused
// but unavoidable buckets, by the speedup bytes of Swiss Tables, and if `flags` contains
// `SizeFlags::CAPACITY`, by empty buckets.
fn fix_set_for_capacity<K>(hash_set: &HashSet<K>, size: usize, flags: SizeFlags) -> usize {
    core::mem::size_of::<HashSet<K>>()
        + size
        + if flags.contains(SizeFlags::CAPACITY) {
            (capacity_to_buckets(hash_set.capacity()).unwrap_or(usize::MAX) - hash_set.len())
                * std::mem::size_of::<K>()
                + capacity_to_buckets(hash_set.capacity()).unwrap_or(usize::MAX)
                    * std::mem::size_of::<u8>()
        } else {
            (capacity_to_buckets(hash_set.len()).unwrap_or(usize::MAX) - hash_set.len())
                * std::mem::size_of::<K>()
                + capacity_to_buckets(hash_set.len()).unwrap_or(usize::MAX)
                    * std::mem::size_of::<u8>()
        }
}

#[cfg(feature = "alloc")]
impl<K: CopyType + MemSize> MemSizeHelper<True> for HashSet<K> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        fix_set_for_capacity(self, std::mem::size_of::<K>() * self.len(), flags)
    }
}

#[cfg(feature = "alloc")]
impl<K: CopyType + MemSize> MemSizeHelper<False> for HashSet<K> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        fix_set_for_capacity(
            self,
            self.iter().map(|x| x.mem_size(flags)).sum::<usize>(),
            flags,
        )
    }
}

/// A helper trait that makes it possible to implement differently
/// the size computation for maps in which keys or values are
/// [`Copy`] types.
///
/// See [`crate::CopyType`] for more information.
pub trait MemSizeHelper2<K: Boolean, V: Boolean> {
    fn mem_size_impl(&self, flags: SizeFlags) -> usize;
}

impl<K: CopyType, V: CopyType> MemSize for HashMap<K, V>
where
    HashMap<K, V>: MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size(&self, flags: SizeFlags) -> usize {
        <HashMap<K, V> as MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>>::mem_size_impl(self, flags)
    }
}

// Add to the given size the space occupied on the stack by the hash map, by the unused
// but unavoidable buckets, by the speedup bytes of Swiss Tables, and if `flags` contains
// `SizeFlags::CAPACITY`, by empty buckets.
fn fix_map_for_capacity<K, V>(hash_map: &HashMap<K, V>, size: usize, flags: SizeFlags) -> usize {
    core::mem::size_of::<HashSet<K>>()
        + size
        + if flags.contains(SizeFlags::CAPACITY) {
            (capacity_to_buckets(hash_map.capacity()).unwrap_or(usize::MAX) - hash_map.len())
                * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
                + capacity_to_buckets(hash_map.capacity()).unwrap_or(usize::MAX)
                    * std::mem::size_of::<u8>()
        } else {
            (capacity_to_buckets(hash_map.len()).unwrap_or(usize::MAX) - hash_map.len())
                * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
                + capacity_to_buckets(hash_map.len()).unwrap_or(usize::MAX)
                    * std::mem::size_of::<u8>()
        }
}

#[cfg(feature = "alloc")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<True, True> for HashMap<K, V> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        fix_map_for_capacity(
            self,
            (std::mem::size_of::<K>() + std::mem::size_of::<V>()) * self.len(),
            flags,
        )
    }
}

#[cfg(feature = "alloc")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<True, False> for HashMap<K, V> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        fix_map_for_capacity(
            self,
            (std::mem::size_of::<K>()) * self.len()
                + self.values().map(|v| v.mem_size(flags)).sum::<usize>(),
            flags,
        )
    }
}

#[cfg(feature = "alloc")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<False, True> for HashMap<K, V> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        fix_map_for_capacity(
            self,
            self.keys().map(|k| k.mem_size(flags)).sum::<usize>()
                + (std::mem::size_of::<V>()) * self.len(),
            flags,
        )
    }
}

#[cfg(feature = "alloc")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<False, False> for HashMap<K, V> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags) -> usize {
        fix_map_for_capacity(
            self,
            self.iter()
                .map(|(k, v)| k.mem_size(flags) + v.mem_size(flags))
                .sum::<usize>(),
            flags,
        )
    }
}

impl<H> CopyType for core::hash::BuildHasherDefault<H> {
    type Copy = True;
}
impl<H> MemSize for core::hash::BuildHasherDefault<H> {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        // it's a phantom hash
        debug_assert_eq!(core::mem::size_of::<Self>(), 0);
        0
    }
}

#[cfg(feature = "std")]
impl CopyType for std::collections::hash_map::RandomState {
    type Copy = True;
}
#[cfg(feature = "std")]
impl MemSize for std::collections::hash_map::RandomState {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl CopyType for core::alloc::Layout {
    type Copy = True;
}
impl MemSize for core::alloc::Layout {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl<T: ?Sized> CopyType for core::ptr::NonNull<T> {
    type Copy = True;
}
impl<T: ?Sized> MemSize for core::ptr::NonNull<T> {
    #[inline(always)]
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        core::mem::size_of::<Self>()
    }
}
