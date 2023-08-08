/*
 * SPDX-FileCopyrightText: 2023 Inria
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

/// A trait to compute recursiely the *overall* size of a structure, as opposed to the
/// *superficial* size returned by [`core::mem::size_of()`].
pub trait MemSize {
    /// Return the (recursively computed) overall
    /// memory size of the structure in bytes.
    fn mem_size(&self) -> usize;
}

/// A trait providing methods to display recursively the content
/// and size of a structure.
///
/// You can derive this trait with `#[derive(MemDbg)]` if all the fields of your structure
/// implement [`MemDbg`]. Note that you will also need to derive [`MemSize`] and [`TypeName`].

pub trait MemDbg: MemSize {
    /// Print debug infos about the structure memory usage, expanding
    /// all levels of nested structures.
    #[cfg(feature = "std")]
    fn mem_dbg(&self) -> core::fmt::Result {
        self.mem_dbg_depth(0, usize::MAX, true, true)
    }

    /// Print debug infos about the structure memory usage, expanding
    /// all levels of nested structures.
    fn mem_dbg_on(&self, writer: &mut impl core::fmt::Write) -> core::fmt::Result {
        self.mem_dbg_depth_on(writer, 0, usize::MAX, None, true, true)
    }

    /// Composite structs should implement this to print their children.
    fn _mem_dbg_rec_on(
        &self,
        _writer: &mut impl core::fmt::Write,
        _depth: usize,
        _max_depth: usize,
        _type_name: bool,
        _humanize: bool,
    ) -> core::fmt::Result {
        Ok(())
    }

    /// Write the data on `writer` debug infos about the structure memory usage, but expanding only
    /// up to `max_depth` levels of nested structures.
    fn mem_dbg_depth_on(
        &self,
        writer: &mut impl core::fmt::Write,
        depth: usize,
        max_depth: usize,
        field_name: Option<&str>,
        type_name: bool,
        humanize: bool,
    ) -> core::fmt::Result {
        if depth > max_depth {
            return Ok(());
        }
        let indent = "  ".repeat(depth);
        writer.write_str(&indent)?;

        if let Some(field_name) = field_name {
            writer.write_str(field_name)?;
        }

        if field_name.is_some() && type_name {
            writer.write_str(" : ")?;
        }

        if type_name {
            writer.write_str(core::any::type_name::<Self>())?;
        }

        if field_name.is_some() | type_name {
            writer.write_str(" = ")?;
        }

        if humanize {
            let (value, uom) = crate::utils::humanize_float(self.mem_size() as f64);
            writer.write_fmt(format_args!("{:>7.3}{}", value, uom,))?;
        } else {
            writer.write_fmt(format_args!("{}", self.mem_size()))?;
        }
        writer.write_char('\n')?;

        self._mem_dbg_rec_on(writer, depth + 1, max_depth, type_name, humanize)
    }

    /// Write to stdout debug infos about the structure memory usage, but expanding only
    /// up to `max_depth` levels of nested structures.
    #[cfg(feature = "std")]
    fn mem_dbg_depth(
        &self,
        depth: usize,
        max_depth: usize,
        type_name: bool,
        humanize: bool,
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
            None,
            type_name,
            humanize,
        )
    }
}
