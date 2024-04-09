//#![feature(offset_of_enum, offset_of_nested)]
/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![allow(dead_code)]

use mem_dbg::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use mem_dbg::*;

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
        c: (u8, String),
    }

    #[derive(MemSize, MemDbg)]
    enum TestEnum {
        Unit,
        Unit2(),
        Unit3 {},
        Unnamed(usize, u8),
        Named { first: usize, second: u8 },
    }

    let b = Vec::with_capacity(100);

    let s = Struct {
        a: TestEnum::Unnamed(0, 16),
        b: Data {
            a: vec![0x42_u8; 700],
            b,
            c: (1, "foo".to_owned()),
        },
        test: -0xbadf00d,
    };

    println!("size:     {}", s.mem_size(SizeFlags::default()));
    println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));
    println!();

    s.mem_dbg(DbgFlags::empty())?;

    println!();

    println!("size:     {}", s.mem_size(SizeFlags::default()));
    println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));
    println!();

    s.mem_dbg(DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE)?;

    #[cfg(feature = "enum_padding")]
    {
        println!();

        println!("size:     {}", s.mem_size(SizeFlags::default()));
        println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));
        println!();

        s.mem_dbg(DbgFlags::empty() | DbgFlags::RUST_LAYOUT)?;
    }
    Ok(())
}
