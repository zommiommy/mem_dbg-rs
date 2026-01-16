#![cfg(feature = "std")]
#![cfg(feature = "derive")]
//! Tests for correctness of memory size reporting using the `cap` allocator.
use cap::Cap;
use std::alloc;
use std::cell::{Cell, RefCell, UnsafeCell, OnceCell};
use std::ffi::OsString;
use std::marker::{PhantomData, PhantomPinned};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::*;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::sync::atomic::*;
use std::time::Duration;
use mem_dbg::*;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

const CORRECTNESS_THRESHOLD: f64 = 0.01; // 1%
const POWERS: &[usize] = &[0, 1, 10, 100, 1_000, 10_000, 100_000, 1_000_000];


macro_rules! check {
    (
        $data: expr
    ) => {
        let start_size = ALLOCATOR.allocated();
        // put it inside a box so the allocator also counts the struct size, 
        // which we include in the reported size
        let data = Box::new({
            $data
        });
        let end_size = ALLOCATOR.allocated();
        let actual_size = end_size - start_size;
        // use capacity because the allocator tracks allocated memory, which includes
        // any overallocation by the data structure
        let reported_size = data.mem_size(SizeFlags::CAPACITY) - 8; // subtract box pointer
        let type_name = core::any::type_name_of_val(&data);
        drop(data);

        // Handle zero-sized types (both sizes are 0)
        if actual_size == 0 && reported_size == 0 {
            return; // Both zero is correct
        }
        let diff_ratio = actual_size.max(reported_size) as f64
            / actual_size.min(reported_size).max(1) as f64
            - 1.0;
        assert!(
            diff_ratio <= CORRECTNESS_THRESHOLD,
            "Size mismatch for {}: actual = {}, reported = {}, diff_ratio = {}",
            type_name,
            actual_size,
            reported_size,
            diff_ratio
        );
    };
}

// Custom struct with derive
#[derive(MemSize, Clone)]
struct SimpleStruct {
    a: u64,
    b: u32,
}

// Struct with Vec field
#[derive(MemSize)]
struct StructWithVec {
    data: Vec<u32>,
}

// Tuple struct
#[derive(MemSize)]
struct TupleStruct(u64, Vec<u8>);

// Enum with various variants
#[derive(MemSize)]
enum TestEnum {
    Unit,
    Single(u64),
    WithVec(Vec<u32>),
    Named { a: u64, b: String },
}

// Nested struct
#[derive(MemSize)]
struct NestedStruct {
    inner: SimpleStruct,
    data: Vec<SimpleStruct>,
}

#[test]
/// sadly they have to be in the same test so we can enforce that they run sequentially
/// 
/// TODO: Arc, Rc
fn test_correctness() {
    // Primitive types
    check!(0_u8);
    check!(0_u16);
    check!(0_u32);
    check!(0_u64);
    check!(0_u128);
    check!(0_usize);
    check!(0_i8);
    check!(0_i16);
    check!(0_i32);
    check!(0_i64);
    check!(0_i128);
    check!(0_isize);
    check!(0.0_f32);
    check!(0.0_f64);
    check!(true);
    check!('a');
    check!(());

    // Atomics
    check!(AtomicBool::new(false));
    check!(AtomicI8::new(0));
    check!(AtomicI16::new(0));
    check!(AtomicI32::new(0));
    check!(AtomicI64::new(0));
    check!(AtomicIsize::new(0));
    check!(AtomicU8::new(0));
    check!(AtomicU16::new(0));
    check!(AtomicU32::new(0));
    check!(AtomicU64::new(0));
    check!(AtomicUsize::new(0));

    // NonZero types
    check!(NonZeroU8::new(1).unwrap());
    check!(NonZeroU16::new(1).unwrap());
    check!(NonZeroU32::new(1).unwrap());
    check!(NonZeroU64::new(1).unwrap());
    check!(NonZeroU128::new(1).unwrap());
    check!(NonZeroUsize::new(1).unwrap());
    check!(NonZeroI8::new(1).unwrap());
    check!(NonZeroI16::new(1).unwrap());
    check!(NonZeroI32::new(1).unwrap());
    check!(NonZeroI64::new(1).unwrap());
    check!(NonZeroI128::new(1).unwrap());
    check!(NonZeroIsize::new(1).unwrap());

    // Phantom types
    check!(PhantomData::<u32>);
    check!(PhantomPinned);

    // Network types
    check!(Ipv4Addr::new(127, 0, 0, 1));
    check!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    check!(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    check!(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
    check!(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0));
    check!(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));

    // Time types
    check!(Duration::from_secs(1));

    // Ranges
    check!(0..10_usize);
    check!(0..=10_usize);
    check!(..10_usize);
    check!(..=10_usize);
    check!(5_usize..);
    
    check!([0_u32; 100]);
    check!([[0_u32; 10]; 10]);

    for &cap in POWERS {
        // Vec with various element types
        check!(Vec::<u8>::with_capacity(cap));
        check!((0..cap).map(|x| x as u8).collect::<Vec<u8>>());
        check!(Vec::<u16>::with_capacity(cap));
        check!((0..cap).map(|x| x as u16).collect::<Vec<u16>>());
        check!(Vec::<u32>::with_capacity(cap));
        check!((0..cap).map(|x| x as u32).collect::<Vec<u32>>());
        check!(Vec::<u64>::with_capacity(cap));
        check!((0..cap).map(|x| x as u64).collect::<Vec<u64>>());
        check!(Vec::<usize>::with_capacity(cap));
        check!((0..cap).map(|x| x as usize).collect::<Vec<usize>>());
        check!(Vec::<String>::with_capacity(cap));
        check!((0..cap).map(|i| i.to_string()).collect::<Vec<String>>());

        // TODO: BTree and Hash collections need improved approximations
        check!(std::collections::BTreeSet::<u32>::from_iter((0..cap).map(|x| x as u32)));
        check!(std::collections::BTreeMap::<u32, u32>::from_iter((0..cap).map(|x| (x as u32, x as u32))));
        check!(std::collections::HashSet::<u32>::from_iter((0..cap).map(|x| x as u32)));
        check!(std::collections::HashMap::<u32, u32>::from_iter((0..cap).map(|x| (x as u32, x as u32))));

        // VecDeque
        check!(std::collections::VecDeque::<u8>::from_iter((0..cap).map(|x| x as u8)));
        check!(std::collections::VecDeque::<u32>::from_iter((0..cap).map(|x| x as u32)));
        check!(std::collections::VecDeque::<u64>::from_iter((0..cap).map(|x| x as u64)));

        // String
        check!(String::with_capacity(cap));
        check!((0..cap).map(|_| 'a').collect::<String>());

        // Box
        check!(Box::new((0..cap).map(|x| x as u32).collect::<Vec<u32>>()));

        // Nested Vec
        check!((0..cap).map(|i| vec![i as u32; 10]).collect::<Vec<Vec<u32>>>());

        // Boxed slice
        check!((0..cap).map(|x| x as u32).collect::<Vec<u32>>().into_boxed_slice());

        // Custom struct with derive
        check!(SimpleStruct { a: 1, b: 2 });
        check!(StructWithVec { data: (0..cap).map(|x| x as u32).collect() });
        check!(TupleStruct(42, (0..cap).map(|x| x as u8).collect()));

        // Vec of custom structs
        check!((0..cap).map(|i| SimpleStruct { a: i as u64, b: i as u32 }).collect::<Vec<SimpleStruct>>());

        // Nested struct
        check!(NestedStruct {
            inner: SimpleStruct { a: 1, b: 2 },
            data: (0..cap).map(|i| SimpleStruct { a: i as u64, b: i as u32 }).collect(),
        });

        // Enum variants
        check!(TestEnum::Unit);
        check!(TestEnum::Single(42));
        check!(TestEnum::WithVec((0..cap).map(|x| x as u32).collect()));
        check!(TestEnum::Named { a: 1, b: "test".repeat(cap) });

        // Tuples
        check!((0_u32, 1_u64));
        check!((0_u32, (0..cap).map(|x| x as u8).collect::<Vec<u8>>()));
        check!(((0..cap).map(|x| x as u32).collect::<Vec<u32>>(), (0..cap).map(|x| x as u64).collect::<Vec<u64>>()));

        // Option
        check!(Some((0..cap).map(|x| x as u32).collect::<Vec<u32>>()));
        check!(Option::<Vec<u32>>::None);

        // Cells
        check!(Cell::new(42_u32));
        check!(RefCell::new((0..cap).map(|x| x as u32).collect::<Vec<u32>>()));
        check!(UnsafeCell::new(42_u32));
        {
            let once: OnceCell<Vec<u32>> = OnceCell::new();
            let _ = once.set((0..cap).map(|x| x as u32).collect::<Vec<u32>>());
            check!(once);
        }

        // Mutex and RwLock
        check!(Mutex::new((0..cap).map(|x| x as u32).collect::<Vec<u32>>()));
        check!(RwLock::new((0..cap).map(|x| x as u32).collect::<Vec<u32>>()));

        // PathBuf and OsString
        check!(PathBuf::from("/tmp/test".repeat(cap.max(1))));
        check!(OsString::from("test".repeat(cap.max(1))));
    }

    // Function pointers (outside loop, no heap allocation)
    fn dummy_fn() -> i32 { 42 }
    fn dummy_fn1(_: i32) -> i32 { 42 }
    fn dummy_fn2(_: i32, _: i32) -> i32 { 42 }
    check!(dummy_fn as fn() -> i32);
    check!(dummy_fn1 as fn(i32) -> i32);
    check!(dummy_fn2 as fn(i32, i32) -> i32);
}