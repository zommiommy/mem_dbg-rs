/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::marker::PhantomPinned;
use core::num::*;
use core::ops::Deref;
use core::{marker::PhantomData, sync::atomic::*};
use std::collections::{HashMap, HashSet};

use crate::impl_mem_size::MemSizeHelper2;
use crate::{impl_mem_size::MemSizeHelper, CopyType, DbgFlags, MemDbg, MemDbgImpl};

// Primitive types, atomic types, ()

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
    AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize,
    NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
    PhantomPinned
}

// Strings

impl MemDbgImpl for str {}

impl MemDbgImpl for String {}

// PhantomData

impl<T: ?Sized> MemDbgImpl for PhantomData<T> {}

// References: we recurse only if FOLLOW_REFS is set

impl<T: ?Sized + MemDbgImpl> MemDbgImpl for &'_ T {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            (**self)._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
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
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            (**self)._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
        } else {
            Ok(())
        }
    }
}

// Option

impl<T: MemDbgImpl> MemDbgImpl for Option<T> {}

// Box

#[cfg(feature = "alloc")]
impl<T: ?Sized + MemDbgImpl> MemDbgImpl for Box<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.as_ref()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(feature = "alloc")]
impl<T: MemDbgImpl> MemDbgImpl for Arc<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.as_ref()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

// Slices

impl<T: CopyType + MemDbgImpl> MemDbgImpl for [T] where [T]: MemSizeHelper<<T as CopyType>::Copy> {}

// Arrays

impl<T: CopyType + MemDbgImpl, const N: usize> MemDbgImpl for [T; N] where
    [T; N]: MemSizeHelper<<T as CopyType>::Copy>
{
}

// Vectors

#[cfg(feature = "alloc")]
impl<T: CopyType + MemDbgImpl> MemDbgImpl for Vec<T> where
    Vec<T>: MemSizeHelper<<T as CopyType>::Copy>
{
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

    // Implement on reversed list, building the tuple type as we cannot expand
    // recursively
    ([($idx:tt => $ty:ident); $( ($nidx:tt => $nty:ident); )*]) => {
        impl_tuples_muncher!([($idx => $ty); $(($nidx => $nty);)* ], ($ty, $($nty,)*));
    };

    // Implement on reversed list and tuple type
    ([($idx:tt => $ty:ident); $( ($nidx:tt => $nty:ident); )*], $tty:ty) => {
        impl<$ty: crate::MemSize + MemDbgImpl, $($nty: crate::MemSize + MemDbgImpl,)*> MemDbgImpl for ($ty, $($nty,)*)  {
            fn _mem_dbg_rec_on(
                &self,
                writer: &mut impl core::fmt::Write,
                total_size: usize,
                max_depth: usize,
                prefix: &mut String,
                _is_last: bool,
                flags: DbgFlags,
            ) -> core::fmt::Result {
                // Compute size of tuple minus one for last-field check.
                let mut _max_idx = $idx;
                $(_max_idx = _max_idx.max($nidx);)*

                let mut id_sizes: Vec<(usize, usize)> = vec![];
                let n;

                {
                    // We use the offset_of information to build the real
                    // space occupied by a field.
                    id_sizes.push(($idx, core::mem::offset_of!($tty, $idx)));
                    $(id_sizes.push(($nidx, core::mem::offset_of!($tty, $nidx)));)*
                    n = id_sizes.len();
                    id_sizes.push((n, core::mem::size_of::<Self>()));
                    // Sort by offset
                    id_sizes.sort_by_key(|x| x.1);
                    // Compute actual sizes
                    for i in 0..n {
                        id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
                    };
                    // Put the candle back
                    id_sizes.sort_by_key(|x| x.0);
                }

                self.$idx.mem_dbg_depth_on(writer, total_size, max_depth, prefix, Some(stringify!($idx)), $idx == _max_idx, id_sizes[$idx].1, flags)?;
                $(
                    self.$nidx.mem_dbg_depth_on(writer, total_size, max_depth, prefix, Some(stringify!($nidx)), $nidx == _max_idx, id_sizes[$nidx].1, flags)?;
                )*
                Ok(())
            }
        }
    };
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

// function pointers cannot recourse
impl<R> MemDbgImpl for fn() -> R {}
impl<A, R> MemDbgImpl for fn(A) -> R {}
impl<A, B, R> MemDbgImpl for fn(A, B) -> R {}
impl<A, B, C, R> MemDbgImpl for fn(A, B, C) -> R {}
impl<A, B, C, D, R> MemDbgImpl for fn(A, B, C, D) -> R {}

// Hash-based containers from the standard library

impl<K: CopyType> MemDbgImpl for HashSet<K> where HashSet<K>: MemSizeHelper<<K as CopyType>::Copy> {}
impl<K: CopyType, V: CopyType> MemDbgImpl for HashMap<K, V> where
    HashMap<K, V>: MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>
{
}

#[cfg(feature = "mmap-rs")]
impl MemDbgImpl for mmap_rs::Mmap {}

#[cfg(feature = "mmap-rs")]
impl MemDbgImpl for mmap_rs::MmapMut {}

impl<H> MemDbgImpl for core::hash::BuildHasherDefault<H> {
    // it's a phantom data so no recursion
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::collections::hash_map::RandomState {
    // it's two u64s, but they are private so can't recurse
}

#[cfg(feature = "std")]
impl MemDbgImpl for core::alloc::Layout {
    // Layout is size + align, but align is unstable so we can't recurse
    // on that, nor implement memdbg or memsize for that :)
}

impl<Idx: MemDbgImpl> MemDbgImpl for core::ops::Range<Idx> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.start
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)?;
        self.end
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

impl<Idx: MemDbgImpl> MemDbgImpl for core::ops::RangeFrom<Idx> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.start
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

impl<Idx: MemDbgImpl> MemDbgImpl for core::ops::RangeInclusive<Idx> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.start()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)?;
        self.end()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

impl<Idx: MemDbgImpl> MemDbgImpl for core::ops::RangeTo<Idx> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.end
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

impl<Idx: MemDbgImpl> MemDbgImpl for core::ops::RangeToInclusive<Idx> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.end
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

impl<T: ?Sized> MemDbgImpl for core::ptr::NonNull<T> {
    // no recursion because we don't follow pointers
}

#[cfg(feature = "rand")]
impl MemDbgImpl for rand::rngs::SmallRng {
    // it's a struct with 4 u64, but it's private so we can't recurse
}

impl<T: MemDbgImpl> MemDbgImpl for core::cell::RefCell<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.borrow()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

impl<T: MemDbgImpl> MemDbgImpl for core::cell::Cell<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        unsafe {
            (*self.as_ptr())._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
        }
    }
}

impl<T: MemDbgImpl> MemDbgImpl for core::cell::UnsafeCell<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        unsafe {
            (*self.get())._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
        }
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::sync::Mutex<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.lock()
            .unwrap()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::sync::RwLock<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.read()
            .unwrap()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::cell::OnceCell<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.get()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::sync::MutexGuard<'_, T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            self.deref()
                ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::sync::RwLockReadGuard<'_, T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            self.deref()
                ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::sync::RwLockWriteGuard<'_, T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            self.deref()
                ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::path::Path {
    // cannot recourse
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::path::PathBuf {
    // cannot recourse
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::ffi::OsStr {
    // cannot recourse
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::ffi::OsString {
    // cannot recourse
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::fs::File {}

#[cfg(feature = "std")]
impl MemDbgImpl for std::fs::OpenOptions {}

#[cfg(feature = "std")]
impl MemDbgImpl for std::fs::Metadata {}

#[cfg(feature = "std")]
impl MemDbgImpl for std::fs::FileTimes {}

#[cfg(feature = "std")]
impl MemDbgImpl for std::fs::FileType {}

#[cfg(feature = "std")]
impl MemDbgImpl for std::fs::Permissions {}

#[cfg(feature = "std")]
impl<T: MemDbgImpl + std::io::Read> MemDbgImpl for std::io::BufReader<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.get_ref()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl + std::io::Write> MemDbgImpl for std::io::BufWriter<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.get_ref()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::io::Cursor<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.get_ref()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A2 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A4 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A8 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A16 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A32 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A64 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A128 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A256 {}
#[cfg(feature = "maligned")]
impl MemDbgImpl for maligned::A512 {}
#[cfg(feature = "maligned")]
impl<A: maligned::Alignment, T: MemDbgImpl> MemDbgImpl for maligned::Aligned<A, T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        self.deref()
            ._mem_dbg_rec_on(writer, total_size, max_depth, prefix, is_last, flags)
    }
}

#[cfg(feature = "half")]
impl MemDbgImpl for half::f16 {}
#[cfg(feature = "half")]
impl MemDbgImpl for half::bf16 {}
