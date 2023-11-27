use core::marker::PhantomData;

use mem_dbg::*;

#[allow(dead_code)]
#[derive(MemSize, MemDbg)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Unamed(usize, u8),
    Named {
        first: usize,
        second: PhantomData<u8>,
    },
}

#[derive(MemSize, MemDbg)]
struct TestConst<const N: usize = 10> {
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
    assert_eq!(8 * std::mem::size_of::<i32>(), v.mem_cap() - v.mem_size());
}

#[test]
fn test_empty_struct() {
    #[derive(MemSize)]
    struct Data {}
    let v = Data {};
    assert_eq!(0, v.mem_size());
    assert_eq!(0, v.mem_cap());
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
    assert_eq!(8 + v.b.mem_size(), v.mem_size());
    assert_eq!(8 + v.b.mem_cap(), v.mem_cap());
}

#[test]
fn test_empty_tuple_struct() {
    #[derive(MemSize)]
    struct Data();
    let v = Data();
    assert_eq!(0, v.mem_size());
    assert_eq!(0, v.mem_cap());
}

#[test]
fn test_tuple_struct() {
    #[derive(MemSize)]
    struct Data(u64, Vec<usize>);
    let mut v = Data(10, Vec::with_capacity(10));
    v.1.push(1);
    v.1.push(2);
    assert_eq!(8 + v.1.mem_size(), v.mem_size());
    assert_eq!(8 + v.1.mem_cap(), v.mem_cap());
}

#[test]
fn test_option() {
    let v = Some(1_usize);
    assert_eq!(v.mem_size(), 2 * core::mem::size_of::<usize>());
    assert_eq!(v.mem_size(), v.mem_cap());
    let v = Some(Some(1_usize));
    assert_eq!(v.mem_size(), 2 * core::mem::size_of::<usize>());
    assert_eq!(v.mem_size(), v.mem_cap());
    let v = Some(Some(Some(1_usize)));
    assert_eq!(v.mem_size(), 2 * core::mem::size_of::<usize>());
    assert_eq!(v.mem_size(), v.mem_cap());
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
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());
    let v = Data::B(1000);
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());
    let d = vec![1, 2, 3, 4, 5];
    let len = d.len();
    let capacity = d.capacity();
    let v = Data::C(1000, d);
    assert_eq!(
        enum_size + core::mem::size_of::<usize>() * len,
        v.mem_size()
    );
    assert_eq!(
        enum_size + core::mem::size_of::<usize>() * capacity,
        v.mem_cap()
    );
}

#[test]
/// <https://github.com/rust-lang/rfcs/issues/1230>
fn test_exotic() {
    // a reference cannot be null, so the compiler should use null as Option's
    // None variant
    let v: Option<&u8> = None;
    assert_eq!(core::mem::size_of::<usize>(), core::mem::size_of::<&u8>());
    assert_eq!(
        core::mem::size_of::<usize>(),
        core::mem::size_of::<Option<&u8>>()
    );
    assert_eq!(core::mem::size_of::<usize>(), v.mem_size());
    assert_eq!(core::mem::size_of::<usize>(), v.mem_cap());

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
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());
    let v = Data1::B;
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());

    let enum_size = core::mem::size_of::<Data2>();
    let v = Data2::A;
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());
    let v = Data2::B(Data1::A);
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());
    let v = Data2::B(Data1::B);
    assert_eq!(enum_size, v.mem_size());
    assert_eq!(enum_size, v.mem_cap());
}