/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![allow(dead_code)]
use std::collections::HashSet;

use mem_dbg::*;

#[derive(Clone, Copy, MemSize, MemDbg)]
#[copy_type]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Unnamed(usize, u8),
    Named { first: usize, second: u8 },
}

#[derive(Clone, Copy, MemSize, MemDbg)]
struct TestMarker;

#[derive(Clone, Copy, MemSize, MemDbg)]
struct TestTuple(usize, u8);

#[derive(MemSize, MemDbg)]
struct Struct<A, B> {
    a: A,
    b: B,
    test: isize,
    s: HashSet<usize>,
}

#[derive(MemSize, MemDbg)]
struct Data<A> {
    a: A,
    b: Vec<i32>,
    c: (usize, String),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut b = Vec::with_capacity(100);
    b.extend(0..10);
    let mut s = HashSet::with_capacity(100);
    s.extend(0..10);

    let s = Struct {
        a: TestEnum::Unnamed(0, 16),
        b: Data {
            a: vec![0x42_u8; 700],
            b,
            c: (1, "foo".to_owned()),
        },
        test: -0xbadf00d,
        s,
    };

    // print the size in bytes of the value
    println!("size:     {}", s.mem_size(SizeFlags::default()));
    println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));

    println!();

    println!("DbgFlags::default():");
    println!();
    s.mem_dbg(DbgFlags::default())?;

    println!();

    println!("DbgFlags::default() | DbgFlags::CAPACITY:");
    println!();
    s.mem_dbg(DbgFlags::default() | DbgFlags::CAPACITY)?;

    println!();

    println!("DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE:");
    println!();
    s.mem_dbg(DbgFlags::default() | DbgFlags::HUMANIZE)?;

    println!();

    println!("DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE:");
    println!();
    s.mem_dbg(DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE)?;

    println!();

    println!("DbgFlags::empty():");
    println!();
    s.mem_dbg(DbgFlags::empty())?;

    println!();

    println!("DbgFlags::HUMANIZE:");
    println!();
    s.mem_dbg(DbgFlags::HUMANIZE)?;

    Ok(())
}
