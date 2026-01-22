/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::marker::PhantomPinned;
use core::num::*;
use core::{marker::PhantomData, sync::atomic::*};

use crate::{CopyType, DbgFlags, HashSet, MemDbgImpl, RefDisplay, impl_mem_size::MemSizeHelper};

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::collections::VecDeque;

/// Implements [`MemDbg`](crate::MemDbg) using the default implementation of
/// [`MemDbgImpl`].
macro_rules! impl_mem_dbg {
     ($($ty:ty),*) => {$(
 impl MemDbgImpl for $ty {}
     )*};
 }

/// Implements `_mem_dbg_depth_on` and `_mem_dbg_rec_on` for pointer-like types.
/// Used by `&T`, `&mut T`, `Rc<T>`, and `Arc<T>`.
///
/// Arguments:
/// - `$flag`: The `DbgFlags` flag to check (e.g., `FOLLOW_REFS` or `FOLLOW_RCS`)
/// - `$get_ptr`: A closure that takes `self` and returns the pointer address as `usize`
macro_rules! impl_mem_dbg_for_deref {
    ($flag:ident, |$self:ident| $get_ptr:expr) => {
        #[allow(clippy::too_many_arguments)]
        fn _mem_dbg_depth_on(
            &self,
            writer: &mut impl core::fmt::Write,
            total_size: usize,
            max_depth: usize,
            prefix: &mut String,
            field_name: Option<&str>,
            is_last: bool,
            padded_size: usize,
            flags: DbgFlags,
            dbg_refs: &mut HashSet<usize>,
        ) -> core::fmt::Result {
            if flags.contains(DbgFlags::$flag) {
                let $self = self;
                let ptr: usize = $get_ptr;
                if dbg_refs.contains(&ptr) {
                    self._mem_dbg_depth_on_impl(
                        writer,
                        total_size,
                        max_depth,
                        prefix,
                        field_name,
                        is_last,
                        padded_size,
                        flags,
                        dbg_refs,
                        RefDisplay::BackReference(ptr),
                    )
                } else {
                    dbg_refs.insert(ptr);
                    self._mem_dbg_depth_on_impl(
                        writer,
                        total_size,
                        max_depth,
                        prefix,
                        field_name,
                        is_last,
                        padded_size,
                        flags,
                        dbg_refs,
                        RefDisplay::FirstEncounter(ptr),
                    )
                }
            } else {
                self._mem_dbg_depth_on_impl(
                    writer,
                    total_size,
                    max_depth,
                    prefix,
                    field_name,
                    is_last,
                    padded_size,
                    flags,
                    dbg_refs,
                    RefDisplay::None,
                )
            }
        }

        fn _mem_dbg_rec_on(
            &self,
            writer: &mut impl core::fmt::Write,
            total_size: usize,
            max_depth: usize,
            prefix: &mut String,
            is_last: bool,
            flags: DbgFlags,
            dbg_refs: &mut HashSet<usize>,
        ) -> core::fmt::Result {
            // Only display inner type when following refs
            if flags.contains(DbgFlags::$flag) {
                let inner = &**self;
                let padded_size = core::mem::size_of_val(inner);
                inner._mem_dbg_depth_on(
                    writer,
                    total_size,
                    max_depth,
                    prefix,
                    None,
                    is_last,
                    padded_size,
                    flags,
                    dbg_refs,
                )
            } else {
                Ok(())
            }
        }
    };
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
    PhantomPinned, str, String
}

impl<T: ?Sized> MemDbgImpl for PhantomData<T> {}

// References: we recurse only if FOLLOW_REFS is set, with deduplication
impl<T: ?Sized + MemDbgImpl> MemDbgImpl for &'_ T {
    impl_mem_dbg_for_deref!(FOLLOW_REFS, |s| *s as *const T as *const () as usize);
}

impl<T: ?Sized + MemDbgImpl> MemDbgImpl for &'_ mut T {
    impl_mem_dbg_for_deref!(FOLLOW_REFS, |s| *s as *const T as *const () as usize);
}

// Option

impl<T: MemDbgImpl> MemDbgImpl for Option<T> {}

// Box

impl<T: ?Sized + MemDbgImpl> MemDbgImpl for Box<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.as_ref()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
    }
}

// Rc

/// This implementation displays the referenced data with deduplication
/// when FOLLOW_RCS is set.
#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::rc::Rc<T> {
    impl_mem_dbg_for_deref!(FOLLOW_RCS, |s| std::rc::Rc::as_ptr(s) as usize);
}

// Arc

/// This implementation displays the referenced data with deduplication
/// when FOLLOW_RCS is set.
#[cfg(feature = "std")]
impl<T: MemDbgImpl> MemDbgImpl for std::sync::Arc<T> {
    impl_mem_dbg_for_deref!(FOLLOW_RCS, |s| std::sync::Arc::as_ptr(s) as usize);
}

// Slices

impl<T: CopyType + MemDbgImpl> MemDbgImpl for [T] where [T]: MemSizeHelper<<T as CopyType>::Copy> {}

// Arrays

impl<T: CopyType + MemDbgImpl, const N: usize> MemDbgImpl for [T; N] where
    [T; N]: MemSizeHelper<<T as CopyType>::Copy>
{
}

// Vectors

impl<T: CopyType + MemDbgImpl> MemDbgImpl for Vec<T> where
    Vec<T>: MemSizeHelper<<T as CopyType>::Copy>
{
}

// VecDeque

impl<T: CopyType + MemDbgImpl> MemDbgImpl for VecDeque<T> where
    VecDeque<T>: MemSizeHelper<<T as CopyType>::Copy>
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
                dbg_refs: &mut HashSet<usize>,
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

                self.$idx._mem_dbg_depth_on(writer, total_size, max_depth, prefix, Some(stringify!($idx)), $idx == _max_idx, id_sizes[$idx].1, flags, dbg_refs)?;
                $(
                    self.$nidx._mem_dbg_depth_on(writer, total_size, max_depth, prefix, Some(stringify!($nidx)), $nidx == _max_idx, id_sizes[$nidx].1, flags, dbg_refs)?;
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

// Function pointers cannot recurse

impl<R> MemDbgImpl for fn() -> R {}
impl<A, R> MemDbgImpl for fn(A) -> R {}
impl<A, B, R> MemDbgImpl for fn(A, B) -> R {}
impl<A, B, C, R> MemDbgImpl for fn(A, B, C) -> R {}
impl<A, B, C, D, R> MemDbgImpl for fn(A, B, C, D) -> R {}

// Hash-based containers from the standard library

#[cfg(feature = "std")]
impl<K: CopyType> MemDbgImpl for std::collections::HashSet<K> where
    std::collections::HashSet<K>: MemSizeHelper<<K as CopyType>::Copy>
{
}
#[cfg(feature = "std")]
impl<K: CopyType, V: CopyType> MemDbgImpl for std::collections::HashMap<K, V> where
    std::collections::HashMap<K, V>:
        crate::impl_mem_size::MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>
{
}

#[cfg(feature = "std")]
impl<T: CopyType> MemDbgImpl for std::collections::BTreeSet<T> where
    std::collections::BTreeSet<T>: MemSizeHelper<<T as CopyType>::Copy>
{
}

#[cfg(feature = "std")]
impl<K: CopyType, V: CopyType> MemDbgImpl for std::collections::BTreeMap<K, V> where
    std::collections::BTreeMap<K, V>:
        crate::impl_mem_size::MemSizeHelper2<<K as CopyType>::Copy, <V as CopyType>::Copy>
{
}

// Hash stuff

#[cfg(feature = "mmap-rs")]
impl_mem_dbg!(mmap_rs::Mmap, mmap_rs::MmapMut);

impl<H> MemDbgImpl for core::hash::BuildHasherDefault<H> {
    // it's a phantom data so no recursion
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::hash::DefaultHasher {
    // Opaque internal state, so no recursion
}

#[cfg(feature = "std")]
impl MemDbgImpl for std::collections::hash_map::RandomState {
    // it's two u64s, but they are private so can't recurse
}

// alloc

#[cfg(feature = "std")]
impl MemDbgImpl for core::alloc::Layout {
    // Layout is size + align, but align is unstable so we can't recurse
    // on that, nor implement memdbg or memsize for that :)
}

// Ranges

impl<Idx: MemDbgImpl> MemDbgImpl for core::ops::Range<Idx> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.start._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )?;
        self.end._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.start._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.start()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )?;
        self.end()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.end._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.end._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
    }
}

impl<T: ?Sized> MemDbgImpl for core::ptr::NonNull<T> {
    // no recursion because we don't follow pointers
}

// Rand crate

#[cfg(feature = "rand")]
impl_mem_dbg!(
    rand::rngs::SmallRng,
    rand::rngs::StdRng,
    rand::rngs::ThreadRng
);

// Cells

impl<T: MemDbgImpl> MemDbgImpl for core::cell::RefCell<T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.borrow()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        // SAFETY: we temporarily take a shared reference to the inner value
        unsafe {
            (&*self.as_ptr())._mem_dbg_rec_on(
                writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
            )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        unsafe {
            // SAFETY: we temporarily take a shared reference to the inner value
            (&*self.get())._mem_dbg_rec_on(
                writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
            )
        }
    }
}

// Mutexes

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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.lock().unwrap()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.read().unwrap()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.get()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        use core::ops::Deref;
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            self.deref()._mem_dbg_rec_on(
                writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
            )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        use core::ops::Deref;
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            self.deref()._mem_dbg_rec_on(
                writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
            )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        use core::ops::Deref;
        if flags.contains(DbgFlags::FOLLOW_REFS) {
            self.deref()._mem_dbg_rec_on(
                writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
            )
        } else {
            Ok(())
        }
    }
}

// Os stuff

#[cfg(feature = "std")]
impl_mem_dbg!(
    std::path::Path,
    std::path::PathBuf,
    std::ffi::OsStr,
    std::ffi::OsString,
    std::fs::File,
    std::fs::OpenOptions,
    std::fs::Metadata,
    std::fs::FileTimes,
    std::fs::FileType,
    std::fs::Permissions
);

// I/O

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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.get_ref()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.get_ref()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
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
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self.get_ref()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
    }
}

// maligned crate

#[cfg(feature = "maligned")]
impl_mem_dbg!(
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
impl<A: maligned::Alignment, T: MemDbgImpl> MemDbgImpl for maligned::Aligned<A, T> {
    fn _mem_dbg_rec_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        is_last: bool,
        flags: DbgFlags,
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        use core::ops::Deref;
        self.deref()._mem_dbg_rec_on(
            writer, total_size, max_depth, prefix, is_last, flags, dbg_refs,
        )
    }
}

// half crate

#[cfg(feature = "half")]
impl_mem_dbg!(half::f16, half::bf16);
