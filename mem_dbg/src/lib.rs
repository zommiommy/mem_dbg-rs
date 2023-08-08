/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![doc = include_str!("../README.md")]
#![deny(unconditional_recursion)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

mod impl_mem_size;
pub(crate) mod utils;

/// A trait to compute recursiely the *overall* size of a structure, as opposed to the
/// *superficial* size returned by [`core::mem::size_of()`].
pub trait MemSize {
    /// Return the (recursively computed) overall
    /// memory size of the structure in bytes.
    fn mem_size(&self) -> usize;
}
