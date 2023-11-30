/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![allow(dead_code)]
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
}

#[derive(MemSize, MemDbg)]
struct Data<A> {
    a: A,
    b: Vec<i32>,
    c: (usize, String),
}

fn main() {
    let b = Vec::with_capacity(100);
    let person = (
        10_usize,
        Struct {
            a: TestEnum::Unnamed(0, 16),
            b: Data {
                a: vec![0x42_u8; 700],
                b,
                c: (1, "foo".to_owned()),
            },
            test: -0xbadf00d,
        },
    );

    // print the size in bytes of the value
    println!("size:     {}", person.mem_size(SizeFlags::default()));
    println!("capacity: {}", person.mem_size(SizeFlags::CAPACITY));

    println!();

    println!("DbgFlags::default():");
    println!();
    person.mem_dbg(DbgFlags::default()).unwrap();

    println!();

    println!("DbgFlags::default() | DbgFlags::HUMANIZE:");
    println!();
    person
        .mem_dbg(DbgFlags::default() | DbgFlags::HUMANIZE)
        .unwrap();

    println!();

    println!("DbgFlags::default() | DbgFlags::CAPACITY:");
    println!();
    person
        .mem_dbg(DbgFlags::default() | DbgFlags::CAPACITY)
        .unwrap();

    println!();

    println!("DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE:");
    println!();
    person
        .mem_dbg(DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE)
        .unwrap();

    println!();

    println!("DbgFlags::empty():");
    println!();
    person.mem_dbg(DbgFlags::empty()).unwrap();

    println!();

    println!("DbgFlags::HUMANIZE:");
    println!();
    person.mem_dbg(DbgFlags::HUMANIZE).unwrap();
}
