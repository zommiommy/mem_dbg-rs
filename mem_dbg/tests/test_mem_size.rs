/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![cfg_attr(feature = "offset_of_enum", feature(offset_of_enum, offset_of_nested))]

use core::marker::PhantomData;
use std::{fmt::Display, sync::atomic::AtomicU64};

use mem_dbg::*;

#[allow(dead_code)]
#[derive(MemSize, MemDbg)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Unnamed(usize, u8),
    Named {
        first: usize,
        second: PhantomData<u8>,
    },
}

#[derive(MemSize, MemDbg)]
struct TestConst<const N: usize = 10>
where
    usize: Display,
{
    a: [u8; N],
}

#[derive(MemSize, MemDbg)]
struct TestMarker;

#[derive(MemSize, MemDbg)]
struct TestTuple(usize, u8);

#[derive(MemSize, MemDbg)]
struct PersonVec<A, B = ()> {
    a: A,
    b: B,
    test: isize,
}

#[derive(MemSize, MemDbg)]
struct Data<A = usize> {
    a: A,
    b: Vec<i32>,
}

#[test]
fn test_vec_capacity() {
    let mut v = vec![Vec::with_capacity(10)];
    v[0].push(1);
    v[0].push(2);
    // We consider the capacity of the inner vector
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY) - v.mem_size(SizeFlags::default()),
        8 * std::mem::size_of::<i32>()
    );
}

#[test]
fn test_vec_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone)]
    struct NewType(usize);

    assert_eq!(
        vec![NewType(1_usize); 10].mem_size(SizeFlags::default()),
        vec![1_usize; 10].mem_size(SizeFlags::default())
    );
}

#[test]
fn test_boxed_slice_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone)]
    struct NewType(usize);

    assert_eq!(
        vec![NewType(1_usize); 10]
            .into_boxed_slice()
            .mem_size(SizeFlags::FOLLOW_REFS),
        vec![1_usize; 10]
            .into_boxed_slice()
            .mem_size(SizeFlags::FOLLOW_REFS)
    );
}

#[test]
fn test_slice_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone)]
    struct NewType(usize);

    assert_eq!(
        vec![NewType(1_usize); 10]
            .into_boxed_slice()
            .as_ref()
            .mem_size(SizeFlags::FOLLOW_REFS),
        vec![1_usize; 10]
            .into_boxed_slice()
            .as_ref()
            .mem_size(SizeFlags::FOLLOW_REFS)
    );
}

#[test]
fn test_array_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone, Copy)]
    struct NewType(usize);

    assert_eq!(
        [NewType(1_usize); 10]
            .as_ref()
            .mem_size(SizeFlags::FOLLOW_REFS),
        [1_usize; 10].as_ref().mem_size(SizeFlags::FOLLOW_REFS)
    );
}

#[test]
fn test_empty_struct() {
    #[derive(MemSize, Clone, Copy)]
    #[copy_type]
    struct Data {}
    let v = Data {};
    assert_eq!(v.mem_size(SizeFlags::default()), 0);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), 0);
}

#[test]
fn test_struct() {
    #[derive(MemSize)]
    struct Data {
        a: u64,
        b: Vec<usize>,
    }
    let mut v = Data {
        a: 10,
        b: Vec::with_capacity(10),
    };
    v.b.push(1);
    v.b.push(2);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        8 + v.b.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        8 + v.b.mem_size(SizeFlags::CAPACITY)
    );
}

#[test]
fn test_empty_tuple_struct() {
    #[derive(MemSize)]
    struct Data();
    let v = Data();
    assert_eq!(v.mem_size(SizeFlags::default()), 0);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), 0);
}

#[test]
fn test_tuple_struct() {
    #[derive(MemSize)]
    struct Data(u64, Vec<usize>);
    let mut v = Data(10, Vec::with_capacity(10));
    v.1.push(1);
    v.1.push(2);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        8 + v.1.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        8 + v.1.mem_size(SizeFlags::CAPACITY),
    );
}

#[test]
fn test_padding() {
    assert_eq!((0_u8, 0_u64).mem_size(SizeFlags::default()), 16);
    #[derive(MemSize)]
    struct TuplePadded((u8, u64));
    let v = TuplePadded((0, 0));
    assert_eq!(v.mem_size(SizeFlags::default()), 16);

    #[derive(MemSize)]
    struct StructPadded(u8, u64);
    let v = StructPadded(0, 0);
    assert_eq!(v.mem_size(SizeFlags::default()), 16);

    #[derive(MemSize)]
    struct StructStructPadded(StructPadded);
    let v = StructStructPadded(StructPadded(0, 0));
    assert_eq!(v.mem_size(SizeFlags::default()), 16);
}

#[test]
fn test_option() {
    let v = Some(1_usize);
    assert_eq!(
        2 * core::mem::size_of::<usize>(),
        v.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default())
    );
    let v = Some(Some(1_usize));
    assert_eq!(
        2 * core::mem::size_of::<usize>(),
        v.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default())
    );
    let v = Some(Some(Some(1_usize)));
    assert_eq!(
        2 * core::mem::size_of::<usize>(),
        v.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default())
    );
}

#[test]
fn test_enum() {
    #[derive(MemSize)]
    #[repr(u8)]
    enum Data {
        A,
        B(u64),
        C(u64, Vec<usize>),
    }

    let enum_size = core::mem::size_of::<Data>();

    let v = Data::A;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data::B(1000);
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let d = vec![1, 2, 3, 4, 5];
    let len = d.len();
    let capacity = d.capacity();
    let v = Data::C(1000, d);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        enum_size + core::mem::size_of::<usize>() * len
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        enum_size + core::mem::size_of::<usize>() * capacity
    );
}

#[test]
/// <https://github.com/rust-lang/rfcs/issues/1230>
fn test_exotic() {
    // A reference cannot be null, so the compiler should use null as Option's
    // None variant
    let v: Option<&u8> = None;
    assert_eq!(core::mem::size_of::<usize>(), core::mem::size_of::<&u8>());
    assert_eq!(
        core::mem::size_of::<usize>(),
        core::mem::size_of::<Option<&u8>>()
    );
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        core::mem::size_of::<usize>()
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<usize>()
    );

    #[derive(MemSize)]
    enum Data1 {
        A,
        B,
    }
    #[derive(MemSize)]
    enum Data2 {
        A,
        B(Data1),
    }

    // nested enums can be flattened IFF they don't have a repr attribute
    assert_eq!(core::mem::size_of::<Data1>(), core::mem::size_of::<Data2>(),);

    let enum_size = core::mem::size_of::<Data1>();
    let v = Data1::A;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data1::B;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);

    let enum_size = core::mem::size_of::<Data2>();
    let v = Data2::A;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data2::B(Data1::A);
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data2::B(Data1::B);
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
}

#[test]
fn test_tuple() {
    assert_eq!((8, 4).mem_size(SizeFlags::default()), 8);
    assert_eq!((8, 4).mem_size(SizeFlags::CAPACITY), 8);
}

#[test]
fn test_atomic() {
    assert_eq!(AtomicU64::new(0).mem_size(SizeFlags::default()), 8);
}

#[test]
fn test_unit() {
    assert_eq!(().mem_size(SizeFlags::default()), 0);
}

#[test]
fn test_phantom() {
    struct Dummy();
    #[derive(MemSize, MemDbg)]
    struct Example<A>(PhantomData<A>);

    Example::<Dummy>(PhantomData)
        .mem_dbg(DbgFlags::default())
        .unwrap();
}

#[test]
fn test_vec_strings() {
    let data = vec![String::new(), String::new()];
    data.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
fn test_array_u8() {
    let data = [0_u8; 10];
    data.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
fn test_array() {
    #[derive(MemSize, MemDbg, Clone, Copy)]
    struct Dummy;
    let data = [Dummy; 10];
    data.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
fn test_indirect_call() {
    #[derive(MemSize, MemDbg)]
    struct Dummy<T>(Vec<T>);

    fn test<T>(data: Vec<T>)
    where
        // this is needed because the type system is not smart enough to infer it
        Vec<T>: MemSize + MemDbgImpl,
    {
        let data = Dummy(data);
        data.mem_dbg(DbgFlags::default()).unwrap();
    }

    test(vec![1, 2, 3, 4, 5]);
}
