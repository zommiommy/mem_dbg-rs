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
    Unamed(usize, u8),
    Named { first: usize, second: u8 },
}

#[derive(Clone, Copy, MemSize, MemDbg)]
struct TestMarker;

#[derive(Clone, Copy, MemSize, MemDbg)]
struct TestTuple(usize, u8);

#[derive(MemSize, MemDbg)]
struct PersonVec<A, B> {
    a: A,
    b: B,
    test: isize,
}

#[derive(MemSize, MemDbg)]
struct Data<A> {
    a: A,
    b: Vec<i32>,
}

fn main() {
    let person = PersonVec {
        a: TestEnum::Unamed(0, 16),
        b: Data {
            a: vec![0x42_u8; 700],
            b: vec![0xbadf00d; 1000],
        },
        test: -0xbadf00d,
    };

    // get the type name of the value
    // print the size in bytes of the value
    println!("mem_size: {}", person.mem_size(SizeFlags::default()));
    println!("mem_size: {}", person.mem_size(SizeFlags::CAPACITY));

    // print the tree of fields and their memory size
    person.mem_dbg(DbgFlags::default()).unwrap();

    let test = TestEnum::Named {
        first: 0x89,
        second: 0x42,
    };
    test.mem_dbg(DbgFlags::default()).unwrap();
}
