/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::marker::{PhantomData, PhantomPinned};
use core::num::*;
use core::sync::atomic::*;

#[cfg(feature = "std")]
use core::ops::Deref;

use crate::{Boolean, False, FlatType, HashMap, MemSize, SizeFlags, True};

#[cfg(not(feature = "std"))]
use alloc::borrow::{Cow, ToOwned};
#[cfg(not(feature = "std"))]
use alloc::collections::{BinaryHeap, LinkedList, VecDeque};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::borrow::{Cow, ToOwned};
#[cfg(feature = "std")]
use std::collections::{BinaryHeap, LinkedList, VecDeque};

/// A basic implementation using [`core::mem::size_of`], setting
/// [`FlatType::Flat`] to the specified type ([`True`] or [`False`]).
macro_rules! impl_size_of {
    ($flat:ty; $($ty:ty),*) => {$(
        impl FlatType for $ty {
            type Flat = $flat;
        }

        impl MemSize for $ty {
            #[inline(always)]
            fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    )*};
}

impl_size_of! {True;
   (), bool, char, f32, f64,
   u8, u16, u32, u64, u128, usize,
   i8, i16, i32, i64, i128, isize,
   AtomicBool,
   AtomicI8, AtomicI16, AtomicI32, AtomicIsize,
   AtomicU8, AtomicU16, AtomicU32, AtomicUsize,
   NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
   NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
   PhantomPinned
}

#[cfg(target_has_atomic = "64")]
impl_size_of! {True; AtomicI64, AtomicU64}

// Strings

impl FlatType for str {
    type Flat = False;
}

impl MemSize for str {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        self.len()
    }
}

impl FlatType for String {
    type Flat = False;
}

impl MemSize for String {
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::CAPACITY) {
                self.capacity()
            } else {
                self.len()
            }
    }
}

// PhantomData

impl<T: ?Sized> FlatType for PhantomData<T> {
    type Flat = True;
}

impl<T: ?Sized> MemSize for PhantomData<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        0
    }
}

// References: we recurse only if FOLLOW_REFS is set, and use the map for deduplication

/// Records the recursive size behind a reference once when `FOLLOW_REFS` is
/// set, keyed by pointer address. A zero-size sentinel is inserted before
/// recursing so that reference cycles terminate instead of recursing forever.
#[inline(always)]
fn record_followed_pointer_size<T: ?Sized + MemSize>(
    value: &T,
    ptr: usize,
    flags: SizeFlags,
    refs: &mut HashMap<usize, usize>,
) {
    if flags.contains(SizeFlags::FOLLOW_REFS) && !refs.contains_key(&ptr) {
        refs.insert(ptr, 0);
        let inner_size = <T as MemSize>::mem_size_rec(value, flags, refs);
        refs.insert(ptr, inner_size);
    }
}

impl<T: ?Sized + MemSize> FlatType for &'_ T {
    type Flat = False;
}

impl<T: ?Sized + MemSize> MemSize for &'_ T {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        record_followed_pointer_size(*self, *self as *const T as *const () as usize, flags, refs);
        core::mem::size_of::<Self>()
    }
}

impl<T: ?Sized + MemSize> FlatType for &'_ mut T {
    type Flat = False;
}

impl<T: ?Sized + MemSize> MemSize for &'_ mut T {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <&'_ T as MemSize>::mem_size_rec(&&**self, flags, refs)
    }
}

// Raw pointers are handles only. Following them would require an unsafe
// validity contract that `MemSize` does not expose.

impl<T: ?Sized> FlatType for *const T {
    type Flat = True;
}

impl<T: ?Sized> MemSize for *const T {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

impl<T: ?Sized> FlatType for *mut T {
    type Flat = True;
}

impl<T: ?Sized> MemSize for *mut T {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Option

impl<T: FlatType> FlatType for Option<T> {
    type Flat = T::Flat;
}

impl<T: MemSize> MemSize for Option<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + self.as_ref().map_or(0, |x| {
                <T as MemSize>::mem_size_rec(x, flags, refs) - core::mem::size_of::<T>()
            })
    }
}

// Result

impl<T: FlatType, E: FlatType> FlatType for Result<T, E> {
    type Flat = <T::Flat as Boolean>::And<E::Flat>;
}

impl<T: MemSize, E: MemSize> MemSize for Result<T, E> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + match self {
                Ok(t) => <T as MemSize>::mem_size_rec(t, flags, refs) - core::mem::size_of::<T>(),
                Err(e) => <E as MemSize>::mem_size_rec(e, flags, refs) - core::mem::size_of::<E>(),
            }
    }
}

// Sum/newtype wrappers only recurse into the active payload.

impl<B: FlatType, C: FlatType> FlatType for core::ops::ControlFlow<B, C> {
    type Flat = <B::Flat as Boolean>::And<C::Flat>;
}

impl<B: MemSize, C: MemSize> MemSize for core::ops::ControlFlow<B, C> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + match self {
                core::ops::ControlFlow::Break(b) => {
                    <B as MemSize>::mem_size_rec(b, flags, refs) - core::mem::size_of::<B>()
                }
                core::ops::ControlFlow::Continue(c) => {
                    <C as MemSize>::mem_size_rec(c, flags, refs) - core::mem::size_of::<C>()
                }
            }
    }
}

impl<T: FlatType> FlatType for core::task::Poll<T> {
    type Flat = T::Flat;
}

impl<T: MemSize> MemSize for core::task::Poll<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + match self {
                core::task::Poll::Ready(t) => {
                    <T as MemSize>::mem_size_rec(t, flags, refs) - core::mem::size_of::<T>()
                }
                core::task::Poll::Pending => 0,
            }
    }
}

impl<T: FlatType> FlatType for core::ops::Bound<T> {
    type Flat = T::Flat;
}

impl<T: MemSize> MemSize for core::ops::Bound<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + match self {
                core::ops::Bound::Included(t) | core::ops::Bound::Excluded(t) => {
                    <T as MemSize>::mem_size_rec(t, flags, refs) - core::mem::size_of::<T>()
                }
                core::ops::Bound::Unbounded => 0,
            }
    }
}

impl<T: FlatType> FlatType for core::cmp::Reverse<T> {
    type Flat = T::Flat;
}

// Reverse<T> is repr(transparent) over T, so we forward straight to the inner
// value, like Cell, UnsafeCell, and Pin.
impl<T: MemSize> MemSize for core::cmp::Reverse<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <T as MemSize>::mem_size_rec(&self.0, flags, refs)
    }
}

// Box: unique ownership, so just recurse directly

impl<T: ?Sized> FlatType for Box<T> {
    type Flat = False;
}

impl<T: ?Sized + MemSize> MemSize for Box<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <T as MemSize>::mem_size_rec(self.as_ref(), flags, refs)
    }
}

// Cow follows borrowed values like references and owned values like their owned
// representation.

impl<B: ToOwned + ?Sized> FlatType for Cow<'_, B> {
    type Flat = False;
}

impl<B> MemSize for Cow<'_, B>
where
    B: ToOwned + MemSize + ?Sized,
    B::Owned: MemSize,
{
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + match self {
                Cow::Borrowed(borrowed) => {
                    <&B as MemSize>::mem_size_rec(borrowed, flags, refs)
                        - core::mem::size_of::<&B>()
                }
                Cow::Owned(owned) => {
                    <B::Owned as MemSize>::mem_size_rec(owned, flags, refs)
                        - core::mem::size_of::<B::Owned>()
                }
            }
    }
}

// `Pin<P>` is `#[repr(transparent)]` over `P`, so it preserves `P`'s sizing
// policy (including `refs` dedup) by forwarding to `P`'s impls.
impl<P: FlatType> FlatType for core::pin::Pin<P> {
    type Flat = P::Flat;
}

impl<P: MemSize> MemSize for core::pin::Pin<P> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        // SAFETY: `Pin<P>` is `#[repr(transparent)]` over `P`, so `&Pin<P>`
        // and `&P` have identical layout. Taking a shared reference to `P`
        // does not move the pointee.
        let pointer = unsafe { &*(self as *const core::pin::Pin<P> as *const P) };
        <P as MemSize>::mem_size_rec(pointer, flags, refs)
    }
}

// Rc: uses map for deduplication when FOLLOW_RCS is set

// Structure used to measure the size of RcInner.
#[repr(C, align(2))]
struct RcInner<T: ?Sized> {
    _strong: core::cell::Cell<usize>,
    _weak: core::cell::Cell<usize>,
    _data: T,
}

/// Records the allocation behind an `Rc`/`Arc`-style shared pointer once when
/// `FOLLOW_RCS` is set, including its control-block header. A zero-size
/// sentinel is inserted before recursing so that cycles terminate. If the same
/// data address was already seen through `FOLLOW_REFS`, upgrade the recorded
/// size so the shared-pointer header is not lost.
#[inline(always)]
fn record_followed_shared_size<T: MemSize>(
    inner: &T,
    ptr: usize,
    inner_header_size: usize,
    flags: SizeFlags,
    refs: &mut HashMap<usize, usize>,
) {
    if !flags.contains(SizeFlags::FOLLOW_RCS) {
        return;
    }

    if let Some(size) = refs.get(&ptr).copied() {
        if size != 0 {
            let inner_size = inner_header_size + <T as MemSize>::mem_size_rec(inner, flags, refs)
                - core::mem::size_of::<T>();
            if let Some(size) = refs.get_mut(&ptr) {
                *size = core::cmp::max(*size, inner_size);
            }
        }
        return;
    }

    refs.insert(ptr, 0);
    let inner_size = inner_header_size + <T as MemSize>::mem_size_rec(inner, flags, refs)
        - core::mem::size_of::<T>();
    refs.insert(ptr, inner_size);
}

#[cfg(feature = "std")]
impl<T> FlatType for std::rc::Rc<T> {
    type Flat = False;
}

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
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        record_followed_shared_size(
            self.as_ref(),
            std::rc::Rc::as_ptr(self) as usize,
            core::mem::size_of::<RcInner<T>>(),
            flags,
            refs,
        );
        core::mem::size_of::<Self>()
    }
}

#[cfg(not(feature = "std"))]
impl<T> FlatType for alloc::rc::Rc<T> {
    type Flat = False;
}

#[cfg(not(feature = "std"))]
impl<T: MemSize> MemSize for alloc::rc::Rc<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        // The pointer address is used only as an identity key for deduplication.
        let ptr = alloc::rc::Rc::as_ptr(self) as usize;
        record_followed_shared_size(
            self.as_ref(),
            ptr,
            core::mem::size_of::<RcInner<T>>(),
            flags,
            refs,
        );
        core::mem::size_of::<Self>()
    }
}

// Weak pointers are handles only. Upgrading them would observe mutable shared
// state and would not establish ownership of the allocation.

#[cfg(feature = "std")]
impl<T: ?Sized> FlatType for std::rc::Weak<T> {
    type Flat = True;
}

#[cfg(feature = "std")]
impl<T: ?Sized> MemSize for std::rc::Weak<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(not(feature = "std"))]
impl<T: ?Sized> FlatType for alloc::rc::Weak<T> {
    type Flat = True;
}

#[cfg(not(feature = "std"))]
impl<T: ?Sized> MemSize for alloc::rc::Weak<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Arc: uses map for deduplication when FOLLOW_RCS is set

#[cfg(feature = "std")]
impl<T> FlatType for std::sync::Arc<T> {
    type Flat = False;
}

// Structure used to measure the size of ArcInner.
#[repr(C, align(2))]
struct ArcInner<T: ?Sized> {
    _strong: core::sync::atomic::AtomicUsize,
    _weak: core::sync::atomic::AtomicUsize,
    _data: T,
}

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
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        record_followed_shared_size(
            self.as_ref(),
            std::sync::Arc::as_ptr(self) as usize,
            core::mem::size_of::<ArcInner<T>>(),
            flags,
            refs,
        );
        core::mem::size_of::<Self>()
    }
}

#[cfg(all(not(feature = "std"), target_has_atomic = "ptr"))]
impl<T> FlatType for alloc::sync::Arc<T> {
    type Flat = False;
}

#[cfg(all(not(feature = "std"), target_has_atomic = "ptr"))]
impl<T: MemSize> MemSize for alloc::sync::Arc<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        // The pointer address is used only as an identity key for deduplication.
        let ptr = alloc::sync::Arc::as_ptr(self) as usize;
        record_followed_shared_size(
            self.as_ref(),
            ptr,
            core::mem::size_of::<ArcInner<T>>(),
            flags,
            refs,
        );
        core::mem::size_of::<Self>()
    }
}

// Weak pointers are handles only; see the `Rc::Weak` implementation.

#[cfg(feature = "std")]
impl<T: ?Sized> FlatType for std::sync::Weak<T> {
    type Flat = True;
}

#[cfg(feature = "std")]
impl<T: ?Sized> MemSize for std::sync::Weak<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(all(not(feature = "std"), target_has_atomic = "ptr"))]
impl<T: ?Sized> FlatType for alloc::sync::Weak<T> {
    type Flat = True;
}

#[cfg(all(not(feature = "std"), target_has_atomic = "ptr"))]
impl<T: ?Sized> MemSize for alloc::sync::Weak<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Containers branch on the Boolean value instead of selecting helper impls for
// `T::Flat`. This also covers non-normalized forms such as `T::Flat::And<False>`.

/// Computes the used element storage for collection-like values.
#[inline(always)]
fn element_storage_size<'a, T, I>(
    iter: I,
    len: usize,
    flags: SizeFlags,
    refs: &mut HashMap<usize, usize>,
) -> usize
where
    T: FlatType + MemSize + 'a,
    I: IntoIterator<Item = &'a T>,
{
    if <<T as FlatType>::Flat as Boolean>::VALUE {
        len * core::mem::size_of::<T>()
    } else {
        iter.into_iter()
            .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs))
            .sum()
    }
}

/// Computes element storage for collections that can reserve spare capacity.
#[inline(always)]
fn capacity_backed_storage_size<'a, T, I>(
    iter: I,
    len: usize,
    capacity: usize,
    flags: SizeFlags,
    refs: &mut HashMap<usize, usize>,
) -> usize
where
    T: FlatType + MemSize + 'a,
    I: IntoIterator<Item = &'a T>,
{
    if <<T as FlatType>::Flat as Boolean>::VALUE {
        let len = if flags.contains(SizeFlags::CAPACITY) {
            capacity
        } else {
            len
        };
        len * core::mem::size_of::<T>()
    } else {
        element_storage_size(iter, len, flags, refs)
            + if flags.contains(SizeFlags::CAPACITY) {
                (capacity - len) * core::mem::size_of::<T>()
            } else {
                0
            }
    }
}

/// Computes recursive heap contributions beyond inline element storage.
#[cfg(feature = "std")]
#[inline(always)]
fn element_heap_extras<'a, T, I>(
    iter: I,
    flags: SizeFlags,
    refs: &mut HashMap<usize, usize>,
) -> usize
where
    T: FlatType + MemSize + 'a,
    I: IntoIterator<Item = &'a T>,
{
    if <<T as FlatType>::Flat as Boolean>::VALUE {
        0
    } else {
        iter.into_iter()
            .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs) - core::mem::size_of::<T>())
            .sum()
    }
}

/// Computes per-entry recursive heap contributions for a map beyond inline
/// key/value storage, in a single pass over the entries. Flat keys or values
/// contribute nothing, and a fully flat map skips the traversal entirely.
#[cfg(feature = "std")]
#[inline(always)]
fn map_heap_extras<'a, K, V, I>(
    iter: I,
    flags: SizeFlags,
    refs: &mut HashMap<usize, usize>,
) -> usize
where
    K: FlatType + MemSize + 'a,
    V: FlatType + MemSize + 'a,
    I: IntoIterator<Item = (&'a K, &'a V)>,
{
    let key_flat = <<K as FlatType>::Flat as Boolean>::VALUE;
    let value_flat = <<V as FlatType>::Flat as Boolean>::VALUE;
    if key_flat && value_flat {
        0
    } else {
        iter.into_iter()
            .map(|(k, v)| {
                let key_extra = if key_flat {
                    0
                } else {
                    <K as MemSize>::mem_size_rec(k, flags, refs) - core::mem::size_of::<K>()
                };
                let value_extra = if value_flat {
                    0
                } else {
                    <V as MemSize>::mem_size_rec(v, flags, refs) - core::mem::size_of::<V>()
                };
                key_extra + value_extra
            })
            .sum()
    }
}

// Slices

impl<T: FlatType> FlatType for [T] {
    type Flat = False;
}

impl<T: FlatType + MemSize> MemSize for [T] {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        element_storage_size(self.iter(), self.len(), flags, refs)
    }
}

// Arrays

impl<T: FlatType, const N: usize> FlatType for [T; N] {
    type Flat = T::Flat;
}

impl<T: FlatType + MemSize, const N: usize> MemSize for [T; N] {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        element_storage_size(self.iter(), N, flags, refs)
    }
}

// Vectors

impl<T> FlatType for Vec<T> {
    type Flat = False;
}

impl<T: FlatType + MemSize> MemSize for Vec<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + capacity_backed_storage_size(self.iter(), self.len(), self.capacity(), flags, refs)
    }
}

// BinaryHeap is Vec-backed, so it follows the same element and capacity rules
// as Vec.

impl<T> FlatType for BinaryHeap<T> {
    type Flat = False;
}

impl<T: FlatType + MemSize> MemSize for BinaryHeap<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + capacity_backed_storage_size(self.iter(), self.len(), self.capacity(), flags, refs)
    }
}

// VecDeque

impl<T> FlatType for VecDeque<T> {
    type Flat = False;
}

impl<T: FlatType + MemSize> MemSize for VecDeque<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + capacity_backed_storage_size(self.iter(), self.len(), self.capacity(), flags, refs)
    }
}

// LinkedList allocates one node per element, with no reserved spare capacity,
// so `SizeFlags::CAPACITY` is equivalent to the default for this container.

/// Layout-equivalent mirror of the private `Node<T>` struct that
/// [`LinkedList<T>`](alloc::collections::LinkedList) heap-allocates for each
/// element. Field types, field order, and `repr` match the standard-library
/// definition:
///
/// ```ignore
/// struct Node<T> {
///     next: Option<NonNull<Node<T>>>,
///     prev: Option<NonNull<Node<T>>>,
///     element: T,
/// }
/// ```
///
/// # Why this type exists
///
/// The [`MemSize`] impl for `LinkedList<T>` needs the per-node heap size,
/// which is `size_of::<Node<T>>()`. `Node<T>` is `pub(crate)` in `std`, so
/// we cannot name it.
///
/// A hand-rolled formula like `2 * size_of::<usize>() + size_of::<T>()` is
/// wrong whenever `align_of::<T>() > align_of::<usize>()`: the compiler
/// inserts padding before `T` and rounds the total size up to a multiple of
/// `align_of::<T>()`. For `T` with `align(32)` and `size_of::<T>() == 32`
/// the formula reports 48 bytes against an actual 64.
///
/// A mirror struct with the same field types and order sidesteps this. The
/// default Rust representation is not formally specified across distinct
/// type definitions, but rustc's current layout algorithm is deterministic
/// for two structs with identical field types in identical order, so
/// `size_of::<LinkedListNode<T>>()` matches `size_of::<Node<T>>()` for any
/// `T`. We accept this implementation detail, same as the `RcInner` and
/// `ArcInner` mirrors used for `Rc<T>` and `Arc<T>`.
///
/// # Stability
///
/// Re-exported through [`crate::LinkedListNode`] only so integration tests
/// can compute expected sizes against the same definition. Not part of the
/// stable public API. If a future standard-library release changes the
/// `LinkedList` node layout, this type must be updated to match.
#[doc(hidden)]
pub struct LinkedListNode<T> {
    _next: Option<core::ptr::NonNull<LinkedListNode<T>>>,
    _prev: Option<core::ptr::NonNull<LinkedListNode<T>>>,
    _element: T,
}

impl<T> FlatType for LinkedList<T> {
    type Flat = False;
}

impl<T: FlatType + MemSize> MemSize for LinkedList<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + if <<T as FlatType>::Flat as Boolean>::VALUE {
                self.len() * core::mem::size_of::<LinkedListNode<T>>()
            } else {
                let per_node_overhead =
                    core::mem::size_of::<LinkedListNode<T>>() - core::mem::size_of::<T>();
                self.iter()
                    .map(|x| <T as MemSize>::mem_size_rec(x, flags, refs) + per_node_overhead)
                    .sum::<usize>()
            }
    }
}

// Tuples

/// Helper macro to build the And chain: and_chain!(A, B, C) => A::And\<B::And\<C>>
macro_rules! and_chain {
    ($single:ty) => { $single };
    ($first:ty, $($rest:ty),+) => { <$first as Boolean>::And<and_chain!($($rest),+)> };
}

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
        impl<$ty: FlatType, $($nty: FlatType,)*> FlatType for ($ty, $($nty,)*) {
            type Flat = and_chain!(<$ty as FlatType>::Flat $(, <$nty as FlatType>::Flat)*);
        }

        impl<$ty: MemSize, $($nty: MemSize,)*> MemSize for ($ty, $($nty,)*) {
            fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
                let mut bytes = ::core::mem::size_of::<Self>();
                bytes += <$ty as MemSize>::mem_size_rec(&self.$idx, flags, refs) - ::core::mem::size_of::<$ty>();
                $( bytes += <$nty as MemSize>::mem_size_rec(&self.$nidx, flags, refs) - ::core::mem::size_of::<$nty>(); )*
                bytes
            }
        }

        impl<$ty, $($nty,)* R> FlatType for fn($ty, $($nty,)*) -> R {
            type Flat = True;
        }

        impl<$ty, $($nty,)* R> MemSize for fn($ty, $($nty,)*) -> R {
            fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
                ::core::mem::size_of::<Self>()
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

impl<R> FlatType for fn() -> R {
    type Flat = True;
}

impl<R> MemSize for fn() -> R {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Ranges

impl<Idx: FlatType> FlatType for core::ops::Range<Idx> {
    type Flat = Idx::Flat;
}

impl<Idx: MemSize> MemSize for core::ops::Range<Idx> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + <Idx as MemSize>::mem_size_rec(&self.start, flags, refs)
            + <Idx as MemSize>::mem_size_rec(&self.end, flags, refs)
            - 2 * core::mem::size_of::<Idx>()
    }
}

impl<Idx: FlatType> FlatType for core::ops::RangeFrom<Idx> {
    type Flat = Idx::Flat;
}

impl<Idx: MemSize> MemSize for core::ops::RangeFrom<Idx> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <Idx as MemSize>::mem_size_rec(&self.start, flags, refs)
            - core::mem::size_of::<Idx>()
    }
}

impl<Idx: FlatType> FlatType for core::ops::RangeInclusive<Idx> {
    type Flat = Idx::Flat;
}

impl<Idx: MemSize> MemSize for core::ops::RangeInclusive<Idx> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + <Idx as MemSize>::mem_size_rec(self.start(), flags, refs)
            + <Idx as MemSize>::mem_size_rec(self.end(), flags, refs)
            - 2 * core::mem::size_of::<Idx>()
    }
}

impl<Idx: FlatType> FlatType for core::ops::RangeTo<Idx> {
    type Flat = Idx::Flat;
}

impl<Idx: MemSize> MemSize for core::ops::RangeTo<Idx> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <Idx as MemSize>::mem_size_rec(&self.end, flags, refs)
            - core::mem::size_of::<Idx>()
    }
}

impl<Idx: FlatType> FlatType for core::ops::RangeToInclusive<Idx> {
    type Flat = Idx::Flat;
}

impl<Idx: MemSize> MemSize for core::ops::RangeToInclusive<Idx> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() + <Idx as MemSize>::mem_size_rec(&self.end, flags, refs)
            - core::mem::size_of::<Idx>()
    }
}

// Rand crate

#[cfg(feature = "rand")]
impl_size_of!(True;
    rand::rngs::SmallRng,
    rand::rngs::ThreadRng,
    rand::rngs::StdRng
);

// Cells

impl<T: FlatType> FlatType for core::cell::RefCell<T> {
    type Flat = T::Flat;
}

impl<T: MemSize> MemSize for core::cell::RefCell<T> {
    /// Returns `size_of::<Self>() + recursive(T)` when no mutable borrow is
    /// active. If a mutable borrow is currently held, the inner `T` cannot
    /// be observed, so this returns only `size_of::<Self>()` and silently
    /// undercounts any heap reachable through `T`. The `MemDbg`
    /// implementation surfaces this with a `<mutably borrowed>` marker.
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        if let Ok(borrow) = self.try_borrow() {
            core::mem::size_of::<Self>() - core::mem::size_of::<T>()
                + <T as MemSize>::mem_size_rec(&*borrow, flags, refs)
        } else {
            // A mutable borrow is active; just report the struct size.
            core::mem::size_of::<Self>()
        }
    }
}

impl<T: ?Sized> FlatType for core::cell::Cell<T> {
    type Flat = True;
}

impl<T: ?Sized> MemSize for core::cell::Cell<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of_val(self)
    }
}

impl<T: FlatType> FlatType for core::cell::OnceCell<T> {
    type Flat = T::Flat;
}

impl<T: MemSize> MemSize for core::cell::OnceCell<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let mut size = core::mem::size_of::<Self>();
        if let Some(t) = self.get() {
            size += <T as MemSize>::mem_size_rec(t, flags, refs) - core::mem::size_of::<T>();
        }
        size
    }
}

#[cfg(feature = "std")]
impl<T: FlatType> FlatType for std::sync::OnceLock<T> {
    type Flat = T::Flat;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::OnceLock<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let mut size = core::mem::size_of::<Self>();
        if let Some(t) = self.get() {
            size += <T as MemSize>::mem_size_rec(t, flags, refs) - core::mem::size_of::<T>();
        }
        size
    }
}

impl<T: ?Sized> FlatType for core::cell::UnsafeCell<T> {
    type Flat = True;
}

impl<T: ?Sized> MemSize for core::cell::UnsafeCell<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of_val(self)
    }
}

// Mutexes

#[cfg(feature = "std")]
impl<T: FlatType> FlatType for std::sync::Mutex<T> {
    type Flat = T::Flat;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::Mutex<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        match self.try_lock() {
            Ok(guard) => {
                core::mem::size_of::<Self>() - core::mem::size_of::<T>()
                    + <T as MemSize>::mem_size_rec(&guard, flags, refs)
            }
            Err(std::sync::TryLockError::Poisoned(err)) => {
                let guard = err.into_inner();
                core::mem::size_of::<Self>() - core::mem::size_of::<T>()
                    + <T as MemSize>::mem_size_rec(&guard, flags, refs)
            }
            Err(std::sync::TryLockError::WouldBlock) => core::mem::size_of::<Self>(),
        }
    }
}

#[cfg(feature = "std")]
impl<T: FlatType> FlatType for std::sync::RwLock<T> {
    type Flat = T::Flat;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLock<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        match self.try_read() {
            Ok(guard) => {
                core::mem::size_of::<Self>() - core::mem::size_of::<T>()
                    + <T as MemSize>::mem_size_rec(&guard, flags, refs)
            }
            Err(std::sync::TryLockError::Poisoned(err)) => {
                let guard = err.into_inner();
                core::mem::size_of::<Self>() - core::mem::size_of::<T>()
                    + <T as MemSize>::mem_size_rec(&guard, flags, refs)
            }
            Err(std::sync::TryLockError::WouldBlock) => core::mem::size_of::<Self>(),
        }
    }
}

/// Helper function to compute the size of a Deref pointer type,
/// such as `MutexGuard`, `RwLockReadGuard`, `RwLockWriteGuard`.
///
/// # Arguments
///
/// * `obj` - The Deref pointer object.
/// * `flags` - The SizeFlags to use for the computation.
#[cfg(feature = "std")]
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
impl<T> FlatType for std::sync::MutexGuard<'_, T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::MutexGuard<'_, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        deref_pointer_size(self, flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> FlatType for std::sync::RwLockReadGuard<'_, T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLockReadGuard<'_, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        deref_pointer_size(self, flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> FlatType for std::sync::RwLockWriteGuard<'_, T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::sync::RwLockWriteGuard<'_, T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        deref_pointer_size(self, flags, refs)
    }
}

// OS stuff

#[cfg(feature = "std")]
impl FlatType for std::path::Path {
    type Flat = False;
}

#[cfg(feature = "std")]
impl MemSize for std::path::Path {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        <std::ffi::OsStr as MemSize>::mem_size_rec(self.as_os_str(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl FlatType for std::path::PathBuf {
    type Flat = False;
}

#[cfg(feature = "std")]
impl MemSize for std::path::PathBuf {
    fn mem_size_rec(&self, flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
            + if flags.contains(SizeFlags::CAPACITY) {
                self.capacity()
            } else {
                self.as_os_str().len()
            }
    }
}

#[cfg(feature = "std")]
impl FlatType for std::ffi::OsStr {
    type Flat = False;
}

#[cfg(feature = "std")]
impl MemSize for std::ffi::OsStr {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        self.as_encoded_bytes().len()
    }
}

#[cfg(feature = "std")]
impl FlatType for std::ffi::OsString {
    type Flat = False;
}

#[cfg(feature = "std")]
impl MemSize for std::ffi::OsString {
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
impl_size_of!(False;
    std::fs::File,
    std::fs::OpenOptions,
    std::fs::Metadata,
    std::fs::FileType,
    std::fs::FileTimes,
    std::fs::Permissions
);

// I/O

#[cfg(feature = "std")]
impl<T> FlatType for std::io::BufReader<T> {
    type Flat = False;
}

// `BufReader<T>` / `BufWriter<T>` count their internal buffer
// (`self.capacity()`) unconditionally, ignoring `SizeFlags::CAPACITY`.
//
// `CAPACITY` distinguishes user-asked memory from allocator-chosen slack:
// `Vec` and friends grow geometrically, so by default we count `len` and
// expose the slack only on demand; `HashMap`/`HashSet` size their bucket
// arrays via load-factor policy, so by default we count filled slots and
// expose the whole table only on demand. BTree node allocation is always
// estimated because there is no exposed length-vs-capacity split for nodes.
//
// Buffered I/O is not slack. The constructor takes the buffer size
// directly (`BufReader::with_capacity(N, _)`, default 8 KiB), so the
// allocation is the user's stated commitment - the same shape as
// `mmap_rs::Mmap`. `buffer().len()` (the bytes currently held) is a
// transient I/O state that swings between 0 and `capacity()` during a
// single read loop; it is not a memory-footprint quantity.
//
// We therefore intentionally treat `CAPACITY` as a no-op for these
// types and always report `capacity()`.
#[cfg(feature = "std")]
impl<T: MemSize + std::io::Read> MemSize for std::io::BufReader<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + self.capacity()
            + <T as MemSize>::mem_size_rec(self.get_ref(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T: std::io::Write> FlatType for std::io::BufWriter<T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: MemSize + std::io::Write> MemSize for std::io::BufWriter<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + self.capacity()
            + <T as MemSize>::mem_size_rec(self.get_ref(), flags, refs)
    }
}

#[cfg(feature = "std")]
impl<T> FlatType for std::io::Cursor<T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: MemSize> MemSize for std::io::Cursor<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>() - core::mem::size_of::<T>()
            + <T as MemSize>::mem_size_rec(self.get_ref(), flags, refs)
    }
}

// IpAddr: these types live in core::net (stable since Rust 1.77), so they
// are available under no_std; std::net re-exports them.
impl_size_of!(True;
    core::net::Ipv4Addr,
    core::net::Ipv6Addr,
    core::net::IpAddr,
    core::net::SocketAddrV4,
    core::net::SocketAddrV6,
    core::net::SocketAddr
);

// Time: Duration lives in core::time; the clock-backed types need std.
impl_size_of!(True; core::time::Duration);

#[cfg(feature = "std")]
impl_size_of!(True;
    std::time::Instant,
    std::time::SystemTime,
    std::time::SystemTimeError
);

// mmap-rs crate

#[cfg(feature = "mmap-rs")]
impl FlatType for mmap_rs::Mmap {
    type Flat = False;
}

#[cfg(feature = "mmap-rs")]
impl MemSize for mmap_rs::Mmap {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        // `Mmap` owns its mapped region and unmaps it on drop, so its bytes
        // belong to the value's footprint regardless of `FOLLOW_REFS`. There
        // is no notion of unused capacity for an mmap, so `CAPACITY` is a
        // no-op too.
        core::mem::size_of::<Self>() + self.len()
    }
}

#[cfg(feature = "mmap-rs")]
impl FlatType for mmap_rs::MmapMut {
    type Flat = False;
}

#[cfg(feature = "mmap-rs")]
impl MemSize for mmap_rs::MmapMut {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        // See `Mmap` above.
        core::mem::size_of::<Self>() + self.len()
    }
}

// Hash-based containers from the standard library

// Hash table allocation layout for the standard library's vendored hashbrown
// 0.16 RawTable. The allocation stores all buckets first, then control bytes
// at an offset rounded up to max(align_of::<T>(), Group::WIDTH).

#[cfg(feature = "std")]
#[cfg(all(
    any(target_arch = "x86_64", target_arch = "x86"),
    target_feature = "sse2",
    not(miri),
))]
const GROUP_WIDTH: usize = 16;
#[cfg(feature = "std")]
#[cfg(all(
    not(all(
        any(target_arch = "x86_64", target_arch = "x86"),
        target_feature = "sse2",
        not(miri),
    )),
    any(
        target_pointer_width = "64",
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "wasm32",
    ),
))]
const GROUP_WIDTH: usize = 8;
#[cfg(feature = "std")]
#[cfg(all(
    not(all(
        any(target_arch = "x86_64", target_arch = "x86"),
        target_feature = "sse2",
        not(miri),
    )),
    not(any(
        target_pointer_width = "64",
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "wasm32",
    )),
))]
const GROUP_WIDTH: usize = 4;

#[cfg(feature = "std")]
#[inline(always)]
fn capacity_to_buckets<T>(cap: usize) -> Option<usize> {
    if cap == 0 {
        return Some(0);
    }

    if cap < 15 {
        let min_cap = match (GROUP_WIDTH, core::mem::size_of::<T>()) {
            (16, 0..=1) => 14,
            (16, 2..=3) => 7,
            (8, 0..=1) => 7,
            _ => 3,
        };
        let cap = core::cmp::max(cap, min_cap);
        return Some(if cap < 4 {
            4
        } else if cap < 8 {
            8
        } else {
            16
        });
    }

    let adjusted_cap = cap.checked_mul(8)? / 7;
    Some(adjusted_cap.next_power_of_two())
}

#[cfg(feature = "std")]
#[inline(always)]
fn hash_table_allocation_size<T>(capacity: usize) -> usize {
    let buckets = match capacity_to_buckets::<T>(capacity) {
        Some(buckets) => buckets,
        None => return usize::MAX,
    };
    if buckets == 0 {
        return 0;
    }

    let ctrl_align = core::cmp::max(core::mem::align_of::<T>(), GROUP_WIDTH);
    let bucket_bytes = match core::mem::size_of::<T>().checked_mul(buckets) {
        Some(bucket_bytes) => bucket_bytes,
        None => return usize::MAX,
    };
    let ctrl_offset = match bucket_bytes.checked_add(ctrl_align - 1) {
        Some(offset) => offset & !(ctrl_align - 1),
        None => return usize::MAX,
    };
    ctrl_offset
        .checked_add(buckets)
        .and_then(|size| size.checked_add(GROUP_WIDTH))
        .unwrap_or(usize::MAX)
}

#[cfg(feature = "std")]
impl<T> FlatType for std::collections::HashSet<T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: FlatType + MemSize> MemSize for std::collections::HashSet<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let heap_extras = element_heap_extras(self.iter(), flags, refs);
        fix_set_for_capacity(self, heap_extras, flags)
    }
}

#[cfg(feature = "std")]
fn fix_set_for_capacity<K>(
    hash_set: &std::collections::HashSet<K>,
    heap_extras: usize,
    flags: SizeFlags,
) -> usize {
    let capacity = if flags.contains(SizeFlags::CAPACITY) {
        hash_set.capacity()
    } else {
        hash_set.len()
    };
    core::mem::size_of::<std::collections::HashSet<K>>()
        + hash_table_allocation_size::<K>(capacity)
        + heap_extras
}

#[cfg(feature = "std")]
impl<K, V> FlatType for std::collections::HashMap<K, V> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<K: FlatType + MemSize, V: FlatType + MemSize> MemSize for std::collections::HashMap<K, V> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let heap_extras = map_heap_extras(self.iter(), flags, refs);
        fix_map_for_capacity(self, heap_extras, flags)
    }
}

#[cfg(feature = "std")]
fn fix_map_for_capacity<K, V>(
    hash_map: &std::collections::HashMap<K, V>,
    heap_extras: usize,
    flags: SizeFlags,
) -> usize {
    let capacity = if flags.contains(SizeFlags::CAPACITY) {
        hash_map.capacity()
    } else {
        hash_map.len()
    };
    core::mem::size_of::<std::collections::HashMap<K, V>>()
        + hash_table_allocation_size::<(K, V)>(capacity)
        + heap_extras
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
    // This is roughly 2 usizes (16 bytes on 64-bit, 8 bytes on 32-bit).
    // This is an approximation as it relies on specific internal implementation details.
    let header_size = 2 * core::mem::size_of::<usize>();

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

    // Estimate the heap usage by walking the levels of the tree.
    //
    // `FILL` is the average number of items per node we assume. After
    // `BTreeMap`'s split-on-overflow rule, sequential insertion settles
    // around `B + 1` items per node (a node splits at `2*B` items into
    // two halves, then refills back toward capacity, so the time-averaged
    // occupancy is just above `B`). Using `B` (the legal minimum)
    // systematically overestimates the node count by ~17%; using
    // `CAPACITY` (the legal maximum) underestimates it.
    //
    // `FILL = B + 1 = 7` was calibrated against the `cap` allocator on a
    // 100M-element `BTreeSet<usize>` (key = `usize`, sequential insertion
    // `0..100_000_000`): real heap 1.96 GB, this formula 1.96 GB
    // (within ~1%); the previous `(len / B) * avg_node_size` formula
    // reported 1.71 GB (-13%), and a fill of plain `B` reports 2.40 GB
    // (+22%). All three sit inside the `test_correctness` 2x bound, so
    // the calibration is the only proof of accuracy at scale.
    //
    //   leaf_count   = ceil(len / FILL)
    //   parent_count = ceil(child_count / (FILL + 1))
    //
    // Each internal node with `FILL` keys has `FILL + 1` child edges, so
    // each non-leaf level has `ceil(prev / (FILL + 1))` internal nodes.
    // Summing the levels gives a far tighter estimate than the prior
    // `(len / B) * avg_node_size`, which silently dropped the root and
    // other internal nodes when `len` was just above `CAPACITY`.
    const FILL: usize = B + 1;
    const INTERNAL_FANOUT: usize = FILL + 1;
    let heap_size = if len <= CAPACITY {
        leaf_size
    } else {
        let leaf_count = len.div_ceil(FILL);
        let mut total = leaf_count * leaf_size;
        let mut level = leaf_count;
        while level > 1 {
            level = level.div_ceil(INTERNAL_FANOUT);
            total += level * internal_size;
        }
        total
    };

    heap_size + item_heap_size
}

#[cfg(feature = "std")]
impl<T> FlatType for std::collections::BTreeSet<T> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<T: FlatType + MemSize> MemSize for std::collections::BTreeSet<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let item_heap_size = element_heap_extras(self.iter(), flags, refs);
        core::mem::size_of::<std::collections::BTreeSet<T>>()
            + estimate_btree_size::<T, ()>(self.len(), item_heap_size)
    }
}

#[cfg(feature = "std")]
impl<K, V> FlatType for std::collections::BTreeMap<K, V> {
    type Flat = False;
}

#[cfg(feature = "std")]
impl<K: FlatType + MemSize, V: FlatType + MemSize> MemSize for std::collections::BTreeMap<K, V> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        let item_heap_size = map_heap_extras(self.iter(), flags, refs);
        core::mem::size_of::<std::collections::BTreeMap<K, V>>()
            + estimate_btree_size::<K, V>(self.len(), item_heap_size)
    }
}

// Hash

impl<H> FlatType for core::hash::BuildHasherDefault<H> {
    type Flat = True;
}
impl<H> MemSize for core::hash::BuildHasherDefault<H> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        // it's a phantom hash
        debug_assert_eq!(core::mem::size_of::<Self>(), 0);
        0
    }
}

#[cfg(feature = "std")]
impl FlatType for std::hash::DefaultHasher {
    type Flat = True;
}

#[cfg(feature = "std")]
// This implementation assumes that DefaultHasher is a fixed-size type
// that does not allocate memory on the heap.
impl MemSize for std::hash::DefaultHasher {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(feature = "std")]
impl FlatType for std::collections::hash_map::RandomState {
    type Flat = True;
}

#[cfg(feature = "std")]
impl MemSize for std::collections::hash_map::RandomState {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// Memory stuff

impl_size_of!(True; core::alloc::Layout);

impl<T: ?Sized> FlatType for core::ptr::NonNull<T> {
    type Flat = True;
}

impl<T: ?Sized> MemSize for core::ptr::NonNull<T> {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of::<Self>()
    }
}

// maligned crate

#[cfg(feature = "maligned")]
impl_size_of!(True;
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
impl<A: maligned::Alignment, T: MemSize + FlatType> FlatType for maligned::Aligned<A, T> {
    type Flat = T::Flat;
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
impl_size_of!(True; half::f16, half::bf16);

// aliasable crate
//
// The aliasable crate provides non-Unique pointer types used to escape noalias.
// Each wrapper is the memory-layout twin of a core/alloc type, so we forward to
// the same sizing policy: AliasableBox mirrors Box, AliasableVec mirrors Vec
// (through the slice helper), AliasableString mirrors String, and AliasableMut
// mirrors &mut T.
//
// For the time being, SizeFlags::CAPACITY is a no-op because aliasable 0.1.3
// exposes no capacity accessor on AliasableVec or AliasableString.
#[cfg(feature = "aliasable")]
mod aliasable {
    use super::*;
    use ::aliasable::AliasableMut;
    use ::aliasable::boxed::AliasableBox;
    use ::aliasable::string::AliasableString;
    use ::aliasable::vec::AliasableVec;
    use core::ops::Deref;

    impl<T: ?Sized> FlatType for AliasableBox<T> {
        type Flat = False;
    }

    impl<T: ?Sized + MemSize> MemSize for AliasableBox<T> {
        fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
            core::mem::size_of::<Self>() + <T as MemSize>::mem_size_rec(self.deref(), flags, refs)
        }
    }

    impl<T> FlatType for AliasableVec<T> {
        type Flat = False;
    }

    impl<T: FlatType + MemSize> MemSize for AliasableVec<T> {
        fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
            core::mem::size_of::<Self>() + <[T] as MemSize>::mem_size_rec(self.deref(), flags, refs)
        }
    }

    impl FlatType for AliasableString {
        type Flat = False;
    }

    impl MemSize for AliasableString {
        fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
            core::mem::size_of::<Self>() + self.deref().len()
        }
    }

    impl<T: ?Sized + MemSize> FlatType for AliasableMut<'_, T> {
        type Flat = False;
    }

    impl<T: ?Sized + MemSize> MemSize for AliasableMut<'_, T> {
        fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
            <&T as MemSize>::mem_size_rec(&self.deref(), flags, refs)
        }
    }
}

// maybe-dangling crate

// MaybeDangling<T> is a repr(transparent) wrapper over T, so it has the same
// layout and sizing policy as T; we forward straight through its Deref, like
// Cell, UnsafeCell, and Pin.
#[cfg(feature = "maybe-dangling")]
impl<T: FlatType> FlatType for maybe_dangling::MaybeDangling<T> {
    type Flat = T::Flat;
}

#[cfg(feature = "maybe-dangling")]
impl<T: MemSize> MemSize for maybe_dangling::MaybeDangling<T> {
    fn mem_size_rec(&self, flags: SizeFlags, refs: &mut HashMap<usize, usize>) -> usize {
        use core::ops::Deref;
        <T as MemSize>::mem_size_rec(self.deref(), flags, refs)
    }
}
