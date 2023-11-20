/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use mem_dbg::*;

#[derive(MemSize)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3{},
    Unamed(usize, u8),
    Named{
        first: usize,
        second: u8,
    }
}

#[derive(MemSize, MemDbg)]
struct TestMarker;

#[derive(MemSize, MemDbg)]
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
    // create a new value to serialize
    let person = PersonVec {
        a: vec![0x89; 600],
        b: Data {
            a: vec![0x42; 700],
            b: vec![0xbadf00d; 2],
        },
        test: -0xbadf00d,
    };

    // get the type name of the value
    // print the size in bytes of the value
    println!("mem_size: {}", person.mem_size());

    // print the tree of fields and their memory size
    person.mem_dbg().unwrap();
}
