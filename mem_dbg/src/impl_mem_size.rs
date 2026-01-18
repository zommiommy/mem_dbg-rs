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

use crate::{Boolean, CopyType, False, MemSize, SizeFlags, True};

// HashMap for pointer deduplication
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(feature = "std")]
/// A basic implementation using [`core::mem::size_of`] for non-[`Copy`] types,
/// setting [`CopyType::Copy`] to [`False`].
macro_rules! impl_size_of {
    ($($ty:ty),*) => {$(
        impl CopyType for $ty {
            type Copy = False;
        }

        impl MemSize for $ty {
            #[inline(always)]
            fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    )*};
}

/// A basic implementation using [`core::mem::size_of`] for [`Copy`] types,
/// setting [`CopyType::Copy`] to [`True`].
macro_rules! impl_copy_size_of {
    ($($ty:ty),*) => {$(
        impl CopyType for $ty {
            type Copy = True;
        }

        impl MemSize for $ty {
            #[inline(always)]
            fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    )*};
}

impl_copy_size_of! {
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
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<usize>() + self.len()
    }
}

impl CopyType for String {
    type Copy = False;
}

impl MemSize for String {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
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
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        0
    }
}

// References: we recurse only if FOLLOW_REFS is set, and use the map for deduplication

impl<T: ?Sized + MemSize> CopyType for &'_ T {
    type Copy = False;
}

impl<T: ?Sized + MemSize> MemSize for &'_ T {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            let ptr = *self as *const T as *const () as usize;
            if !refs.contains_key(&ptr) {
                let inner_size = <T as MemSize>::mem_size_rec(*self, flags, refs);
                refs.insert(ptr, inner_size);
            }
            core::mem::size_of::<Self>()
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
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <&'_ T as MemSize>::mem_size_rec(&&**self, flags, refs)
    }
}

// Option

impl<T: CopyType + MemSize> CopyType for Option<T> {
    type Copy = T::Copy;
}

impl<T: MemSize> MemSize for Option<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + self.as_ref().map_or(0, |x| {
                <T as MemSize>::mem_size_rec(x, flags, refs) - core::mem::size_of::<T>()
            })
    }
}

// Box: unique ownership, so just recurse directly

impl<T: ?Sized> CopyType for Box<T> {
    type Copy = False;
}

impl<T: ?Sized + MemSize> MemSize for Box<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <T as MemSize>::mem_size_rec(self.as_ref(), flags, refs)
    }
}

// Structure used to occupy the equivalent space of RcInner/ArcInner in std
#[doc(hidden)]
#[cfg(feature = "std")]
struct RcInner<T: ?Sized> {
    _strong: std::cell::Cell<usize>,
    _weak: std::cell::Cell<usize>,
    _data: T,
}

// Rc: uses map for deduplication when FOLLOW_RC is set

/// This implementation is based on the assumption that `Rc<T>` is
/// implemented as follows:
/// ```ignore
/// pub struct Rc<T: ?Sized, A: Allocator = Global> {
///     ptr: NonNull<RcInner<T>>,
///     phantom: PhantomData<RcInner<T>>,
///     alloc: A,
/// }
///
/// struct RcInner<T: ?Sized> {
///     strong: Cell<usize>,
///     weak: Cell<usize>,
///     data: T,
/// }
/// ```
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::rc::Rc<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::FOLLOW_RC) {
            let ptr = std::rc::Rc::as_ptr(self) as usize;
            if !refs.contains_key(&ptr) {
                // Size of RcInner (header) + inner value's recursive size
                let inner_size = core::mem::size_of::<RcInner<T>>()
                    + <T as MemSize>::mem_size_rec(self.as_ref(), flags, refs)
                    - core::mem::size_of::<T>();
                refs.insert(ptr, inner_size);
            }
        }
        core::mem::size_of::<Self>()
    }
}

// Arc: uses map for deduplication when FOLLOW_RC is set

/// This implementation is based on the assumption that `Arc<T>` is
/// implemented as follows:
/// ```ignore
///
/// pub struct Arc<T: ?Sized, A: Allocator = Global> {
///     ptr: NonNull<ArcInner<T>>,
///     phantom: PhantomData<ArcInner<T>>,
///     alloc: A,
/// }
///
/// struct ArcInner<T: ?Sized> {
///     strong: Atomic<usize>,
///     weak: Atomic<usize>,
///     data: T,
/// }
/// ```
#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::Arc<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::FOLLOW_RC) {
            let ptr = std::sync::Arc::as_ptr(self) as usize;
            if !refs.contains_key(&ptr) {
                // Size of ArcInner (header) + inner value's recursive size
                let inner_size = core::mem::size_of::<RcInner<T>>()
                    + <T as MemSize>::mem_size_rec(self.as_ref(), flags, refs)
                    - core::mem::size_of::<T>();
                refs.insert(ptr, inner_size);
            }
        }
        core::mem::size_of::<Self>()
    }
}

/// A helper trait that makes it possible to implement differently
/// the size computation for arrays, vectors, and slices of
/// [`Copy`] types.
///
/// See [`crate::CopyType`] for more information.
pub trait MemSizeHelper<T: Boolean> {
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize;
}

// Slices

impl<T: CopyType> MemSize for [T]
where
    [T]: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <[T] as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags, refs)
    }
}

impl<T: CopyType + MemSize> MemSizeHelper<True> for [T] {
    #[inline(always)]
    fn mem_size_impl(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of_val(self)
    }
}

impl<T: CopyType + MemSize> MemSizeHelper<False> for [T] {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        self.iter()
            .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs))
            .sum::<usize>()
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
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <[T; N] as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags, refs)
    }
}

impl<T: MemSize, const N: usize> MemSizeHelper<True> for [T; N] {
    #[inline(always)]
    fn mem_size_impl(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl<T: MemSize, const N: usize> MemSizeHelper<False> for [T; N] {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + self
                .iter()
                .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs) - core::mem::size_of::<T>())
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
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <Vec<T> as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags, refs)
    }
}

impl<T: CopyType + MemSize> MemSizeHelper<True> for Vec<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>() + self.capacity() * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>() + self.len() * core::mem::size_of::<T>()
        }
    }
}

impl<T: CopyType + MemSize> MemSizeHelper<False> for Vec<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>()
                + self
                    .iter()
                    .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs))
                    .sum::<usize>()
                + (self.capacity() - self.len()) * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>()
                + self
                    .iter()
                    .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs))
                    .sum::<usize>()
        }
    }
}

// VecDeque

impl<T> CopyType for VecDeque<T> {
    type Copy = False;
}

impl<T: CopyType> MemSize for VecDeque<T>
where
    VecDeque<T>: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <VecDeque<T> as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(self, flags, refs)
    }
}

impl<T: CopyType + MemSize> MemSizeHelper<True> for VecDeque<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>() + self.capacity() * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>() + self.len() * core::mem::size_of::<T>()
        }
    }
}

impl<T: CopyType + MemSize> MemSizeHelper<False> for VecDeque<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::CAPACITY) {
            core::mem::size_of::<Self>()
                + self
                    .iter()
                    .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs))
                    .sum::<usize>()
                + (self.capacity() - self.len()) * core::mem::size_of::<T>()
        } else {
            core::mem::size_of::<Self>()
                + self
                    .iter()
                    .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs))
                    .sum::<usize>()
        }
    }
}

// Tuples

macro_rules! impl_tuples_muncher {
    () => {};

    (($idx:tt => $ty:ident), $(($i:tt => $t:ident),)*) => {
        // Pass to list reversal
        impl_tuples_muncher!([($idx => $ty);] $(($i => $t),)*);
        // Recurse on tail
        impl_tuples_muncher!($(($i => $t),)*);
    };

    // List reversal
    ([$(($accIdx: tt => $accTyp: ident);)+]  ($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
        impl_tuples_muncher!([($idx => $typ); $(($accIdx => $accTyp); )*] $( ($nidx => $ntyp), ) *);
    };

    // Implement on reversed list
    ([($idx:tt => $ty:ident); $( ($nidx:tt => $nty:ident); )*]) => {
        impl<$ty, $($nty,)*> CopyType for ($ty, $($nty,)*)  {
            type Copy = False;
		}

		impl<$ty: MemSize, $($nty: MemSize,)*> MemSize for ($ty, $($nty,)*)
        {
            #[inline(always)]
            fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
                let mut bytes = core::mem::size_of::<Self>();
                bytes += <$ty as MemSize>::mem_size_rec(&self.$idx, flags, refs) - core::mem::size_of::<$ty>();
                $( bytes += <$nty as MemSize>::mem_size_rec(&self.$nidx, flags, refs) - core::mem::size_of::<$nty>(); )*
                bytes
            }
        }

        impl<$ty, $($nty,)* R> CopyType for fn($ty, $($nty,)*) -> R {
            type Copy = True;
        }

        impl<$ty, $($nty,)* R> MemSize for fn($ty, $($nty,)*) -> R {
            #[inline(always)]
            fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    }

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

// Functions

impl<R> CopyType for fn() -> R {
    type Copy = True;
}

impl<R> MemSize for fn() -> R {
    #[inline(always)]
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Ranges

impl<Idx: CopyType> CopyType for core::ops::Range<Idx> {
    type Copy = True;
}

impl<Idx: MemSize> MemSize for core::ops::Range<Idx> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + <Idx as MemSize>::mem_size_rec(&self.start, flags, refs)
            + <Idx as MemSize>::mem_size_rec(&self.end, flags, refs)
            - 2 * core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeFrom<Idx> {
    type Copy = True;
}

impl<Idx: MemSize> MemSize for core::ops::RangeFrom<Idx> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <Idx as MemSize>::mem_size_rec(&self.start, flags, refs)
            - core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeInclusive<Idx> {
    type Copy = True;
}

impl<Idx: MemSize> MemSize for core::ops::RangeInclusive<Idx> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + <Idx as MemSize>::mem_size_rec(self.start(), flags, refs)
            + <Idx as MemSize>::mem_size_rec(self.end(), flags, refs)
            - 2 * core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeTo<Idx> {
    type Copy = True;
}

impl<Idx: MemSize> MemSize for core::ops::RangeTo<Idx> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <Idx as MemSize>::mem_size_rec(&self.end, flags, refs)
            - core::mem::size_of::<Idx>()
    }
}

impl<Idx: CopyType> CopyType for core::ops::RangeToInclusive<Idx> {
    type Copy = True;
}

impl<Idx: MemSize> MemSize for core::ops::RangeToInclusive<Idx> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <Idx as MemSize>::mem_size_rec(&self.end, flags, refs)
            - core::mem::size_of::<Idx>()
    }
}

// Rand crate

#[cfg(feature = "rand")]
impl_copy_size_of!(
    rand::rngs::SmallRng,
    rand::rngs::ThreadRng,
    rand::rngs::StdRng
);

// Cells

impl<T: CopyType> CopyType for core::cell::RefCell<T> {
    type Copy = T::Copy;
}

impl<T: MemSize> MemSize for core::cell::RefCell<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(&self.borrow(), flags, refs)
    }
}

impl<T: CopyType> CopyType for core::cell::Cell<T> {
    type Copy = T::Copy;
}

impl<T: MemSize> MemSize for core::cell::Cell<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        // SAFETY: we temporarily take a shared reference to the inner value
        unsafe { <T as MemSize>::mem_size_rec(&*self.as_ptr(), flags, refs) }
    }
}

impl<T: CopyType> CopyType for core::cell::OnceCell<T> {
    type Copy = T::Copy;
}

impl<T: MemSize> MemSize for core::cell::OnceCell<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let mut size = core::mem::size_of::<Self>();
        if let Some(t) = self.get() {
            size += <T as MemSize>::mem_size_rec(t, flags, refs) - core::mem::size_of::<T>();
        }
        size
    }
}

impl<T: CopyType> CopyType for core::cell::UnsafeCell<T> {
    type Copy = T::Copy;
}

impl<T: MemSize> MemSize for core::cell::UnsafeCell<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        // SAFETY: we temporarily take a shared reference to the inner value
        unsafe { <T as MemSize>::mem_size_rec(&*self.get(), flags, refs) }
    }
}

// Mutexes

#[cfg(feature = "std")]
impl<T> CopyType for std::sync::Mutex<T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::Mutex<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(&self.lock().unwrap(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> CopyType for std::sync::RwLock<T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLock<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(&self.read().unwrap(), flags, refs)
    }
}

/// Helper function to compute the size of a Deref pointer type,
/// such as `MutexGuard`, `RwLockReadGuard`, `RwLockWriteGuard`.
///
/// # Arguments
///
/// * `obj` - The Deref pointer object.
/// * `flags` - The SizeFlags to use for the computation.
#[inline(always)]
fn deref_pointer_size<M>(obj: &M, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize
where
    M: Deref<Target: MemSize + Sized>,
{
    core::mem::size_of::<M>()
        + if flags.contains(SizeFlags::FOLLOW_REFS) {
            <M::Target as MemSize>::mem_size_rec(obj.deref(), flags, refs)
                - core::mem::size_of::<M::Target>()
        } else {
            0
        }
}

#[cfg(feature = "std")]
impl<T> CopyType for std::sync::MutexGuard<'_, T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::MutexGuard<'_, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        deref_pointer_size(self, flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> CopyType for std::sync::RwLockReadGuard<'_, T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLockReadGuard<'_, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        deref_pointer_size(self, flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> CopyType for std::sync::RwLockWriteGuard<'_, T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLockWriteGuard<'_, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        deref_pointer_size(self, flags, refs)
    }
}

// OS stuff

#[cfg(feature = "std")]
impl CopyType for std::path::Path {
    type Copy = False;
}

#[cfg(feature = "std")]
impl MemSize for std::path::Path {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <std::ffi::OsStr as MemSize>::mem_size_rec(self.as_os_str(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl CopyType for std::path::PathBuf {
    type Copy = False;
}

#[cfg(feature = "std")]
impl MemSize for std::path::PathBuf {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::CAPACITY) {
                self.capacity()
            } else {
                <std::ffi::OsStr as MemSize>::mem_size_rec(self.as_os_str(), flags, refs)
            }
    }
}

#[cfg(feature = "std")]
impl CopyType for std::ffi::OsStr {
    type Copy = False;
}

#[cfg(feature = "std")]
impl MemSize for std::ffi::OsStr {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        if flags.contains(SizeFlags::FOLLOW_REFS) {
            // OsStr is unsized, so we can only return the length of the data
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
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        // OsString is like String - it has heap-allocated data
        // We use len() by default, and capacity() with CAPACITY flag
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::CAPACITY) {
                self.capacity()
            } else {
                self.len()
            }
    }
}

#[cfg(feature = "std")]
impl_size_of!(
    std::fs::File,
    std::fs::OpenOptions,
    std::fs::Metadata,
    std::fs::FileType,
    std::fs::FileTimes,
    std::fs::Permissions
);

// I/O

#[cfg(feature = "std")]
impl<T> CopyType for std::io::BufReader<T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize + std::io::Read> MemSize for std::io::BufReader<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(self.get_ref(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T: MemSize + std::io::Write> CopyType for std::io::BufWriter<T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize + std::io::Write> MemSize for std::io::BufWriter<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(self.get_ref(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> CopyType for std::io::Cursor<T> {
    type Copy = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::io::Cursor<T> {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(self.get_ref(), flags, refs)
    }
}

// IpAddr
#[cfg(feature = "std")]
impl_copy_size_of!(
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::IpAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6,
    std::net::SocketAddr
);

// Time
#[cfg(feature = "std")]
impl_copy_size_of!(
    std::time::Duration,
    std::time::Instant,
    std::time::SystemTime,
    std::time::SystemTimeError
);

// mmap-rs crate

#[cfg(feature = "mmap-rs")]
impl CopyType for mmap_rs::Mmap {
    type Copy = False;
}

#[cfg(feature = "mmap-rs")]
impl MemSize for mmap_rs::Mmap {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::FOLLOW_REFS) {
                self.len()
            } else {
                0
            }
    }
}

#[cfg(feature = "mmap-rs")]
impl CopyType for mmap_rs::MmapMut {
    type Copy = False;
}

#[cfg(feature = "mmap-rs")]
impl MemSize for mmap_rs::MmapMut {
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::FOLLOW_REFS) {
                self.len()
            } else {
                0
            }
    }
}

// Hash-based containers from the standard library

// If the standard library changes load factor, this code will have to change
// accordingly.

// Group width for Swiss Tables (hashbrown). This depends on SIMD support:
// - x86_64 with SSE2: 16 bytes
// - Other platforms (ARM64 NEON, generic): 8 bytes
#[cfg(feature = "std")]
#[cfg(all(
    target_arch = "x86_64",
    any(target_feature = "sse2", target_env = "msvc")
))]
const GROUP_WIDTH: usize = 16;
#[cfg(feature = "std")]
#[cfg(not(all(
    target_arch = "x86_64",
    any(target_feature = "sse2", target_env = "msvc")
)))]
const GROUP_WIDTH: usize = 8;

// Straight from hashbrown
#[cfg(feature = "std")]
fn capacity_to_buckets(cap: usize) -> Option<usize> {
    if cap == 0 {
        return Some(0);
    }
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

#[cfg(feature = "std")]
impl<T: CopyType> MemSize for std::collections::HashSet<T>
where
    std::collections::HashSet<T>: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <std::collections::HashSet<T> as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(
            self, flags, refs,
        )
    }
}

#[cfg(feature = "std")]
// Add to the given size the space occupied on the stack by the hash set, by the unused
// but unavoidable buckets, by the speedup bytes of Swiss Tables, and if `flags` contains
// `SizeFlags::CAPACITY`, by empty buckets.
fn fix_set_for_capacity<K>(
    hash_set: &std::collections::HashSet<K>,
    size: usize,
    flags: SizeFlags,
) -> usize {
    let capacity = if flags.contains(SizeFlags::CAPACITY) {
        hash_set.capacity()
    } else {
        hash_set.len()
    };
    let buckets = capacity_to_buckets(capacity).unwrap_or(usize::MAX);
    core::mem::size_of::<std::collections::HashSet<K>>()
        + size
        + (buckets - hash_set.len()) * core::mem::size_of::<K>()
        + buckets * core::mem::size_of::<u8>()
        + if buckets > 0 { GROUP_WIDTH } else { 0 }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize> MemSizeHelper<True> for std::collections::HashSet<K> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        fix_set_for_capacity(self, core::mem::size_of::<K>() * self.len(), flags)
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize> MemSizeHelper<False> for std::collections::HashSet<K> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        fix_set_for_capacity(
            self,
            self.iter()
                .map(|x| <K as MemSize>::mem_size_rec(x, flags, refs))
                .sum::<usize>(),
            flags,
        )
    }
}

#[cfg(feature = "std")]
/// A helper trait that makes it possible to implement differently
/// the size computation for maps in which keys or values are
/// [`Copy`] types.
///
/// See [`crate::CopyType`] for more information.
pub trait MemSizeHelper2<K: Boolean, V: Boolean> {
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize;
}

#[cfg(feature = "std")]
impl<K: CopyType, V: CopyType> MemSize for std::collections::HashMap<K, V>
where
    std::collections::HashMap<K, V>: MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <std::collections::HashMap<K, V> as MemSizeHelper2<
            <K as CopyType>::Copy,
            <V as CopyType>::Copy,
        >>::mem_size_impl(self, flags, refs)
    }
}

#[cfg(feature = "std")]
// Add to the given size the space occupied on the stack by the hash map, by the unused
// but unavoidable buckets, by the speedup bytes of Swiss Tables, and if `flags` contains
// `SizeFlags::CAPACITY`, by empty buckets.
fn fix_map_for_capacity<K, V>(
    hash_map: &std::collections::HashMap<K, V>,
    size: usize,
    flags: SizeFlags,
) -> usize {
    let capacity = if flags.contains(SizeFlags::CAPACITY) {
        hash_map.capacity()
    } else {
        hash_map.len()
    };
    let buckets = capacity_to_buckets(capacity).unwrap_or(usize::MAX);
    core::mem::size_of::<std::collections::HashMap<K, V>>()
        + size
        + (buckets - hash_map.len()) * (core::mem::size_of::<K>() + core::mem::size_of::<V>())
        + buckets * core::mem::size_of::<u8>()
        + if buckets > 0 { GROUP_WIDTH } else { 0 }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<True, True>
    for std::collections::HashMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        fix_map_for_capacity(
            self,
            (core::mem::size_of::<K>() + core::mem::size_of::<V>()) * self.len(),
            flags,
        )
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<True, False>
    for std::collections::HashMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        fix_map_for_capacity(
            self,
            (core::mem::size_of::<K>()) * self.len()
                + self
                    .values()
                    .map(|v| <V as MemSize>::mem_size_rec(v, flags, refs))
                    .sum::<usize>(),
            flags,
        )
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<False, True>
    for std::collections::HashMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        fix_map_for_capacity(
            self,
            self.keys()
                .map(|k| <K as MemSize>::mem_size_rec(k, flags, refs))
                .sum::<usize>()
                + (core::mem::size_of::<V>()) * self.len(),
            flags,
        )
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<False, False>
    for std::collections::HashMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        fix_map_for_capacity(
            self,
            self.iter()
                .map(|(k, v)| {
                    <K as MemSize>::mem_size_rec(k, flags, refs)
                        + <V as MemSize>::mem_size_rec(v, flags, refs)
                })
                .sum::<usize>(),
            flags,
        )
    }
}

/// Estimates the heap-allocated memory of a BTree-based container.
///
/// The standard library's `BTreeMap` and `BTreeSet` use a B-Tree with a
/// branching factor `B` (currently 6). The exact memory size depends on
/// the node layout, which includes headers, keys, values, and child pointers.
///
/// This function estimates the memory usage by simulating the node layout
/// and assuming an average node occupancy.
#[cfg(feature = "std")]
fn estimate_btree_size<K, V>(len: usize, item_heap_size: usize) -> usize {
    if len == 0 {
        return 0;
    }
    // The branching factor `B` in the standard library is currently 6.
    // This determines the maximum capacity of a node: 2 * B - 1.
    const B: usize = 6;
    const CAPACITY: usize = 2 * B - 1; // 11

    // Layout calculations:
    let ptr_size = core::mem::size_of::<usize>();

    // A node header typically contains parent pointers and metadata (height, len).
    // On 64-bit systems, this is roughly 16 bytes (2 usizes or similar).
    // This is an approximation as it relies on specific internal implementation details.
    let header_size = 16;

    // Helper to align a size to the next multiple of `align`.
    let align_up = |size: usize, align: usize| -> usize { (size + align - 1) & !(align - 1) };

    let k_size = core::mem::size_of::<K>();
    let v_size = core::mem::size_of::<V>();

    // Leaf Node Layout:
    // [Header] -> [Padding] -> [Keys] -> [Values]
    let mut leaf_size = header_size;
    // Align for keys
    leaf_size = align_up(leaf_size, core::mem::align_of::<K>());
    leaf_size += k_size * CAPACITY;
    // Align for values
    leaf_size = align_up(leaf_size, core::mem::align_of::<V>());
    leaf_size += v_size * CAPACITY;

    // Internal Node Layout:
    // [Leaf Node Part] -> [Padding] -> [Child Pointers]
    // Internal nodes store keys and values (like leaves) plus child pointers.
    let mut internal_size = leaf_size;
    // Align for child pointers (usize)
    internal_size = align_up(internal_size, core::mem::align_of::<usize>());
    internal_size += ptr_size * (CAPACITY + 1);

    // Calculate weighted average node size.
    // We heavily weight leaf nodes as they contain the majority of data.
    // Ratio is approximately B leaves per 1 internal node.
    let avg_node_size = (leaf_size * B + internal_size) / (B + 1);

    // Estimate total heap usage:
    // If the tree fits in a single node (len <= CAPACITY), it's just one leaf.
    // Otherwise, we estimate the number of nodes based on average occupancy.
    let heap_size = if len <= CAPACITY {
        leaf_size
    } else {
        // Approximate node count assuming each node is roughly half full (B items).
        (len / B) * avg_node_size
    };

    heap_size + item_heap_size
}

#[cfg(feature = "std")]
impl<T: CopyType> MemSize for std::collections::BTreeSet<T>
where
    std::collections::BTreeSet<T>: MemSizeHelper<<T as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <std::collections::BTreeSet<T> as MemSizeHelper<<T as CopyType>::Copy>>::mem_size_impl(
            self, flags, refs,
        )
    }
}

#[cfg(feature = "std")]
impl<T: CopyType + MemSize> MemSizeHelper<True> for std::collections::BTreeSet<T> {
    #[inline(always)]
    fn mem_size_impl(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<std::collections::BTreeSet<T>>()
            + estimate_btree_size::<T, ()>(self.len(), 0)
    }
}

#[cfg(feature = "std")]
impl<T: CopyType + MemSize> MemSizeHelper<False> for std::collections::BTreeSet<T> {
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<std::collections::BTreeSet<T>>()
            + estimate_btree_size::<T, ()>(
                self.len(),
                self.iter()
                    .map(|x| {
                        <T as MemSize>::mem_size_rec(x, flags, refs) - core::mem::size_of::<T>()
                    })
                    .sum::<usize>(),
            )
    }
}

#[cfg(feature = "std")]
impl<K: CopyType, V: CopyType> MemSize for std::collections::BTreeMap<K, V>
where
    std::collections::BTreeMap<K, V>: MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>,
{
    #[inline(always)]
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <std::collections::BTreeMap<K, V> as MemSizeHelper2<
            <K as CopyType>::Copy,
            <V as CopyType>::Copy,
        >>::mem_size_impl(self, flags, refs)
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<True, True>
    for std::collections::BTreeMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<std::collections::BTreeMap<K, V>>()
            + estimate_btree_size::<K, V>(self.len(), 0)
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<True, False>
    for std::collections::BTreeMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<std::collections::BTreeMap<K, V>>()
            + estimate_btree_size::<K, V>(
                self.len(),
                self.values()
                    .map(|v| {
                        <V as MemSize>::mem_size_rec(v, flags, refs) - core::mem::size_of::<V>()
                    })
                    .sum::<usize>(),
            )
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<False, True>
    for std::collections::BTreeMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<std::collections::BTreeMap<K, V>>()
            + estimate_btree_size::<K, V>(
                self.len(),
                self.keys()
                    .map(|k| {
                        <K as MemSize>::mem_size_rec(k, flags, refs) - core::mem::size_of::<K>()
                    })
                    .sum::<usize>(),
            )
    }
}

#[cfg(feature = "std")]
impl<K: CopyType + MemSize, V: CopyType + MemSize> MemSizeHelper2<False, False>
    for std::collections::BTreeMap<K, V>
{
    #[inline(always)]
    fn mem_size_impl(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<std::collections::BTreeMap<K, V>>()
            + estimate_btree_size::<K, V>(
                self.len(),
                self.iter()
                    .map(|(k, v)| {
                        <K as MemSize>::mem_size_rec(k, flags, refs) - core::mem::size_of::<K>()
                            + <V as MemSize>::mem_size_rec(v, flags, refs)
                            - core::mem::size_of::<V>()
                    })
                    .sum::<usize>(),
            )
    }
}

// Hash

impl<H> CopyType for core::hash::BuildHasherDefault<H> {
    type Copy = True;
}
impl<H> MemSize for core::hash::BuildHasherDefault<H> {
    #[inline(always)]
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        // it's a phantom hash
        debug_assert_eq!(core::mem::size_of::<Self>(), 0);
        0
    }
}

#[cfg(feature = "std")]
impl CopyType for std::hash::DefaultHasher {
    type Copy = True;
}

#[cfg(feature = "std")]
// This implementation assumes that DefaultHasher is a fixed-size type
// that does not allocate memory on the heap.
impl MemSize for std::hash::DefaultHasher {
    #[inline(always)]
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(feature = "std")]
impl CopyType for std::collections::hash_map::RandomState {
    type Copy = True;
}

#[cfg(feature = "std")]
impl MemSize for std::collections::hash_map::RandomState {
    #[inline(always)]
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Memory stuff

impl_copy_size_of!(core::alloc::Layout);

impl<T: ?Sized> CopyType for core::ptr::NonNull<T> {
    type Copy = True;
}

impl<T: ?Sized> MemSize for core::ptr::NonNull<T> {
    #[inline(always)]
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// maligned crate

#[cfg(feature = "maligned")]
impl_copy_size_of!(
    maligned::A2,
    maligned::A4,
    maligned::A8,
    maligned::A16,
    maligned::A32,
    maligned::A64,
    maligned::A128,
    maligned::A256,
    maligned::A512
);

#[cfg(feature = "maligned")]
impl<A: maligned::Alignment, T: MemSize> CopyType for maligned::Aligned<A, T> {
    type Copy = True;
}

#[cfg(feature = "maligned")]
impl<A: maligned::Alignment, T: MemSize> MemSize for maligned::Aligned<A, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        use core::ops::Deref;
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(self.deref(), flags, refs)
    }
}

// half crate

#[cfg(feature = "half")]
impl_copy_size_of!(half::f16, half::bf16);
