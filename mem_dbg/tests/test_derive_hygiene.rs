/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Hygiene tests for the derive macros: the generated code must compile
//! without the prelude (no unqualified `Vec`, `String`, `vec!`, `Some`,
//! `Ok`, or trait-method calls relying on prelude imports). This mirrors
//! the situation of a `no_std` user crate, which has no `String`/`Vec` in
//! scope.

#![no_implicit_prelude]
#![cfg(feature = "derive")]
#![cfg_attr(feature = "offset_of_enum", feature(offset_of_enum))]
#![allow(dead_code)]

#[derive(::mem_dbg::MemSize, ::mem_dbg::MemDbg)]
#[mem_size(rec)]
struct Named {
    a: usize,
    b: ::std::vec::Vec<u8>,
    c: ::std::string::String,
}

#[derive(::mem_dbg::MemSize, ::mem_dbg::MemDbg)]
#[mem_size(rec)]
struct Tuple(usize, ::std::vec::Vec<u8>);

#[derive(::mem_dbg::MemSize, ::mem_dbg::MemDbg)]
#[mem_size(flat)]
struct Unit;

#[derive(::mem_dbg::MemSize, ::mem_dbg::MemDbg)]
#[mem_size(flat)]
struct Zst {
    a: (),
    b: (),
}

#[derive(::mem_dbg::MemSize, ::mem_dbg::MemDbg)]
#[mem_size(rec)]
struct Generic<T> {
    a: T,
    b: usize,
}

#[derive(::mem_dbg::MemSize, ::mem_dbg::MemDbg)]
#[mem_size(rec)]
enum Mixed {
    Unit,
    Named { x: usize, y: ::std::string::String },
    Unnamed(u32, ::std::vec::Vec<u8>),
}

#[test]
fn test_derived_impls_work_without_prelude() {
    use ::core::default::Default;
    use ::mem_dbg::{DbgFlags, MemDbg, MemSize, SizeFlags};

    let named = Named {
        a: 1,
        b: ::std::vec![1, 2, 3],
        c: ::std::string::ToString::to_string("abc"),
    };
    ::core::assert_eq!(
        named.mem_size(SizeFlags::default()),
        ::core::mem::size_of::<Named>() + 3 + 3
    );

    let mixed = Mixed::Unnamed(7, ::std::vec![1, 2, 3]);
    ::core::assert_eq!(
        mixed.mem_size(SizeFlags::default()),
        ::core::mem::size_of::<Mixed>() + 3
    );

    let mut output = ::std::string::String::new();
    named.mem_dbg_on(&mut output, DbgFlags::default()).unwrap();
    ::core::assert!(!output.is_empty());

    output.clear();
    mixed.mem_dbg_on(&mut output, DbgFlags::default()).unwrap();
    ::core::assert!(output.contains("Unnamed"));
}
