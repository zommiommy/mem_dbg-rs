/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../../README.md")]
#![deny(unconditional_recursion)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

// Re-export mem_dbg_derive conditional to the "derive" feature.
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "derive")]
extern crate mem_dbg_derive;
#[cfg(feature = "derive")]
pub use mem_dbg_derive::{MemDbg, MemSize};

mod impl_mem_dbg;
mod impl_mem_size;
pub(crate) mod utils;

/// A trait to compute recursively the overall size and capacity of a structure, as opposed to the
/// stack size returned by [`core::mem::size_of()`].
///
/// The trait provides two functions, [`MemSize::mem_size`] and [`MemSize::mem_capacity`], which
/// return the memory used, and the memory allocated, respectively.
pub trait MemSize {
    /// Return the (recursively computed) overall
    /// memory size of the structure in bytes.
    ///
    /// Size does not include memory allocated but not
    /// used: for example, in the case of a vector this function
    /// calls [`Vec::len`] rather than [`Vec::capacity`].
    fn mem_size(&self) -> usize;

    /// Return the (recursively computed) overall
    /// memory capacity of the structure in bytes.
    ///
    /// Capacity includes also memory allocated but not
    /// used: for example, in the case of a vector this function
    /// calls [`Vec::capacity`] rather than [`Vec::len`].
    ///
    /// The default trait implementation returns the same value as [`MemSize::mem_size`].
    fn mem_capacity(&self) -> usize {
        self.mem_size()
    }
}

/// A trait providing methods to display recursively the content
/// and size of a structure.
///
/// You can derive this trait with `#[derive(MemDbg)]` if all the fields of your structure
/// implement [`MemDbg`]. Note that you will also need to derive [`MemSize`].
pub trait MemDbg: MemDbgImpl {
    /// Print debug infos about the structure memory usage, expanding
    /// all levels of nested structures.
    #[cfg(feature = "std")]
    #[inline(always)]
    fn mem_dbg(&self) -> core::fmt::Result {
        self.mem_dbg_depth(0, usize::MAX, true, true, false)
    }

    /// Print debug infos about the structure memory usage, expanding
    /// all levels of nested structures.
    #[inline(always)]
    fn mem_dbg_on(&self, writer: &mut impl core::fmt::Write) -> core::fmt::Result {
        self.mem_dbg_depth_on(writer, 0, usize::MAX, Some("$ROOT"), true, true, false)
    }

    /// Write to stdout debug infos about the structure memory usage, but expanding only
    /// up to `max_depth` levels of nested structures.
    #[cfg(feature = "std")]
    #[inline(always)]
    fn mem_dbg_depth(
        &self,
        depth: usize,
        max_depth: usize,
        type_name: bool,
        humanize: bool,
        is_last: bool,
    ) -> core::fmt::Result {
        struct Wrapper(std::io::Stdout);
        impl core::fmt::Write for Wrapper {
            #[inline(always)]
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                use std::io::Write;
                self.0
                    .lock()
                    .write(s.as_bytes())
                    .map_err(|_| core::fmt::Error)
                    .map(|_| ())
            }
        }
        self.mem_dbg_depth_on(
            &mut Wrapper(std::io::stdout()),
            depth,
            max_depth,
            Some("$ROOT"),
            type_name,
            humanize,
            is_last,
        )
    }

    /// Write the data on `writer` debug infos about the structure memory usage, but expanding only
    /// up to `max_depth` levels of nested structures.
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn mem_dbg_depth_on(
        &self,
        writer: &mut impl core::fmt::Write,
        depth: usize,
        max_depth: usize,
        field_name: Option<&str>,
        type_name: bool,
        humanize: bool,
        is_last: bool,
    ) -> core::fmt::Result {
        if depth > max_depth {
            return Ok(());
        }
        let real_size = self.mem_size();
        if humanize {
            let (value, uom) = crate::utils::humanize_float(real_size as f64);
            if uom == " B" {
                writer.write_fmt(format_args!("{:>5} B ", real_size))?;
            } else {
                let mut precision = 4;
                let a = value.abs();
                if a >= 100.0 {
                    precision = 1;
                } else if a >= 10.0 {
                    precision = 2;
                } else if a >= 1.0 {
                    precision = 3;
                }
                writer.write_fmt(format_args!("{0:>4.1$} {2}", value, precision, uom))?;
            }
        } else {
            writer.write_fmt(format_args!("{:>5} B ", real_size))?;
        }

        writer.write_char(' ')?;

        let indent = "│".repeat(depth.saturating_sub(1));
        writer.write_str(&indent)?;
        if depth > 0 {
            if is_last {
                writer.write_char('╰')?;
            } else {
                writer.write_char('├')?;
            }
            writer.write_char('╴')?;
        }

        if let Some(field_name) = field_name {
            writer.write_fmt(format_args!("{:}", field_name))?;
        }

        if type_name {
            writer.write_str(" ")?;
            writer.write_fmt(format_args!("{:}", core::any::type_name::<Self>()))?;
        }

        writer.write_char('\n')?;

        self._mem_dbg_rec_on(writer, depth + 1, max_depth, type_name, humanize, false)
    }
}

/// Implement [`MemDbg`] for all types that implement [`MemDbgImpl`].
/// This is done so that no one can change the implementation of [`MemDbg`],
/// this ensures consistency in printing.
impl<T: MemDbgImpl> MemDbg for T {}

/// Inner trait used to implement [`MemDbg`].
/// This trait should not be implemented by users, but they should use the
/// [`MemDbg`] derive macro instead.
pub trait MemDbgImpl: MemSize {
    #[inline(always)]
    /// Composite structs should implement this to print their children.
    fn _mem_dbg_rec_on(
        &self,
        _writer: &mut impl core::fmt::Write,
        _depth: usize,
        _max_depth: usize,
        _type_name: bool,
        _humanize: bool,
        _is_last: bool,
    ) -> core::fmt::Result {
        Ok(())
    }
}
