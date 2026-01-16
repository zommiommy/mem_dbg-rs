#![cfg(all(feature = "std", feature = "derive"))]
#![allow(dead_code)]

use core::alloc::Layout;
use core::cell::{Cell, OnceCell, RefCell, UnsafeCell};
use core::hash::BuildHasherDefault;
use core::marker::{PhantomData, PhantomPinned};
use core::num::*;
use core::ops::*;
use core::ptr::NonNull;
use core::sync::atomic::*;
use mem_dbg::*;
use std::collections::hash_map::{DefaultHasher, RandomState};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

static STATIC_STR: &str = "static";

#[derive(MemSize, MemDbg)]
struct AllTypesStruct<'a> {
    // Unit and primitives
    unit: (),
    boolean: bool,
    character: char,
    f32_val: f32,
    f64_val: f64,

    // Unsigned integers
    u8_val: u8,
    u16_val: u16,
    u32_val: u32,
    u64_val: u64,
    u128_val: u128,
    usize_val: usize,

    // Signed integers
    i8_val: i8,
    i16_val: i16,
    i32_val: i32,
    i64_val: i64,
    i128_val: i128,
    isize_val: isize,

    // Atomics
    atomic_bool: AtomicBool,
    atomic_i8: AtomicI8,
    atomic_i16: AtomicI16,
    atomic_i32: AtomicI32,
    atomic_i64: AtomicI64,
    atomic_isize: AtomicIsize,
    atomic_u8: AtomicU8,
    atomic_u16: AtomicU16,
    atomic_u32: AtomicU32,
    atomic_u64: AtomicU64,
    atomic_usize: AtomicUsize,

    // NonZero
    nz_i8: NonZeroI8,
    nz_i16: NonZeroI16,
    nz_i32: NonZeroI32,
    nz_i64: NonZeroI64,
    nz_i128: NonZeroI128,
    nz_isize: NonZeroIsize,
    nz_u8: NonZeroU8,
    nz_u16: NonZeroU16,
    nz_u32: NonZeroU32,
    nz_u64: NonZeroU64,
    nz_u128: NonZeroU128,
    nz_usize: NonZeroUsize,

    // Phantom types
    phantom_pinned: PhantomPinned,
    phantom_data: PhantomData<i32>,

    // Strings
    string: String,

    // References
    reference: &'a str,
    mut_reference: &'a mut i32,

    // Option
    opt_some: Option<i32>,
    opt_none: Option<String>,

    // Box
    boxed: Box<u64>,

    // Vec and arrays
    vec: Vec<i32>,
    vec_str: Vec<String>,
    array: [u8; 10],
    array_str: [String; 2],

    // Slices (via Box)
    slice: Box<[u32]>,
    slice_str: Box<[String]>,

    // Tuples
    tuple2: (i32, String),
    tuple3: (i32, String, f64),
    tuple4: (i32, String, f64, bool),
    tuple5: (i32, String, f64, bool, u8),

    // Ranges
    range: Range<usize>,
    range_from: RangeFrom<usize>,
    range_inclusive: RangeInclusive<usize>,
    range_to: RangeTo<usize>,
    range_to_inclusive: RangeToInclusive<usize>,

    // Cells
    cell: Cell<i32>,
    ref_cell: RefCell<i32>,
    unsafe_cell: UnsafeCell<i32>,
    once_cell: OnceCell<String>,

    // Sync primitives
    mutex: Mutex<i32>,
    rw_lock: RwLock<String>,

    // Collections
    hash_set: HashSet<i32>,
    hash_set_str: HashSet<String>,
    hash_map: HashMap<String, i32>,
    hash_map_copy: HashMap<i32, i32>,
    btree_set: BTreeSet<i32>,
    btree_set_str: BTreeSet<String>,
    btree_map: BTreeMap<String, i32>,
    btree_map_copy: BTreeMap<i32, i32>,
    vec_deque: VecDeque<u32>,

    // Hash builders
    build_hasher: BuildHasherDefault<DefaultHasher>,
    random_state: RandomState,

    // Path types
    path_buf: PathBuf,
    path: &'a Path,
    os_string: OsString,
    os_str: &'a OsStr,

    // Function pointers
    fn_ptr0: fn() -> i32,
    fn_ptr1: fn(i32) -> i32,
    fn_ptr2: fn(i32, i32) -> i32,
    fn_ptr3: fn(u32, u64, i32) -> bool,
    fn_ptr4: fn(u32, u64, i32, f64) -> bool,

    // Layout
    layout: Layout,

    // NonNull
    non_null: NonNull<i32>,

    // I/O types
    buf_reader: BufReader<File>,
    buf_writer: BufWriter<File>,
    cursor: Cursor<Vec<u8>>,
}

// Helper functions for function pointers
fn fn_ptr_0() -> i32 {
    42
}
fn fn_ptr_1(x: i32) -> i32 {
    x + 1
}
fn fn_ptr_2(x: i32, y: i32) -> i32 {
    x + y
}
fn fn_ptr_3(_a: u32, _b: u64, _c: i32) -> bool {
    true
}
fn fn_ptr_4(_a: u32, _b: u64, _c: i32, _d: f64) -> bool {
    true
}

#[test]
fn test_all_types() {
    let mut hash_set = HashSet::new();
    hash_set.insert(1);
    hash_set.insert(2);

    let mut hash_set_str = HashSet::new();
    hash_set_str.insert("a".to_string());

    let mut hash_map = HashMap::new();
    hash_map.insert("key1".to_string(), 100);

    let mut hash_map_copy = HashMap::new();
    hash_map_copy.insert(1, 1);

    let mut btree_set = BTreeSet::new();
    btree_set.insert(1);
    btree_set.insert(2);

    let mut btree_set_str = BTreeSet::new();
    btree_set_str.insert("a".to_string());

    let mut btree_map = BTreeMap::new();
    btree_map.insert("key1".to_string(), 100);

    let mut btree_map_copy = BTreeMap::new();
    btree_map_copy.insert(1, 1);
    let mut vec_deque: VecDeque<u32> = VecDeque::new();
    vec_deque.push_back(10);

    let once_cell = OnceCell::new();
    once_cell.set("initialized".to_string()).unwrap();

    let os_string = OsString::from("os string");
    let os_str = os_string.as_os_str();

    let path_buf = PathBuf::from("/tmp/test");
    let path = path_buf.as_path();
    let mut data0 = 0;
    let mut_ref0 = &mut data0;
    let mut data1 = 0;
    let mut_ref1 = &mut data1;

    let all_types = AllTypesStruct {
        unit: (),
        boolean: true,
        character: 'A',
        f32_val: 3.78,
        f64_val: 9.81,

        u8_val: 8,
        u16_val: 16,
        u32_val: 32,
        u64_val: 64,
        u128_val: 128,
        usize_val: 1024,

        i8_val: -8,
        i16_val: -16,
        i32_val: -32,
        i64_val: -64,
        i128_val: -128,
        isize_val: -1024,

        atomic_bool: AtomicBool::new(true),
        atomic_i8: AtomicI8::new(8),
        atomic_i16: AtomicI16::new(16),
        atomic_i32: AtomicI32::new(32),
        atomic_i64: AtomicI64::new(64),
        atomic_isize: AtomicIsize::new(1024),
        atomic_u8: AtomicU8::new(8),
        atomic_u16: AtomicU16::new(16),
        atomic_u32: AtomicU32::new(32),
        atomic_u64: AtomicU64::new(64),
        atomic_usize: AtomicUsize::new(1024),

        nz_i8: NonZeroI8::new(1).unwrap(),
        nz_i16: NonZeroI16::new(1).unwrap(),
        nz_i32: NonZeroI32::new(1).unwrap(),
        nz_i64: NonZeroI64::new(1).unwrap(),
        nz_i128: NonZeroI128::new(1).unwrap(),
        nz_isize: NonZeroIsize::new(1).unwrap(),
        nz_u8: NonZeroU8::new(1).unwrap(),
        nz_u16: NonZeroU16::new(1).unwrap(),
        nz_u32: NonZeroU32::new(1).unwrap(),
        nz_u64: NonZeroU64::new(1).unwrap(),
        nz_u128: NonZeroU128::new(1).unwrap(),
        nz_usize: NonZeroUsize::new(1).unwrap(),

        phantom_pinned: PhantomPinned,
        phantom_data: PhantomData,

        string: "Hello".to_string(),

        reference: STATIC_STR,
        mut_reference: mut_ref0,

        opt_some: Some(42),
        opt_none: None,

        boxed: Box::new(12345),

        vec: vec![1, 2, 3, 4, 5],
        vec_str: vec!["v1".to_string(), "v2".to_string()],
        array: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        array_str: ["s1".to_string(), "s2".to_string()],

        slice: vec![10, 20, 30].into_boxed_slice(),
        slice_str: vec!["s1".to_string()].into_boxed_slice(),

        tuple2: (42, "answer".to_string()),
        tuple3: (1, "two".to_string(), 3.0),
        tuple4: (1, "two".to_string(), 3.0, true),
        tuple5: (1, "two".to_string(), 3.0, true, 5),

        range: 0..10,
        range_from: 5..,
        range_inclusive: 0..=10,
        range_to: ..10,
        range_to_inclusive: ..=10,

        cell: Cell::new(100),
        ref_cell: RefCell::new(200),
        unsafe_cell: UnsafeCell::new(300),
        once_cell,

        mutex: Mutex::new(400),
        rw_lock: RwLock::new("locked".to_string()),

        hash_set,
        hash_set_str,
        hash_map,
        hash_map_copy,
        btree_set,
        btree_set_str,
        btree_map,
        btree_map_copy,
        vec_deque,

        build_hasher: BuildHasherDefault::<DefaultHasher>::default(),
        random_state: RandomState::new(),

        path_buf: PathBuf::from("/tmp/test"),
        path,
        os_string: OsString::from("os string"),
        os_str,

        fn_ptr0: fn_ptr_0,
        fn_ptr1: fn_ptr_1,
        fn_ptr2: fn_ptr_2,
        fn_ptr3: fn_ptr_3,
        fn_ptr4: fn_ptr_4,

        layout: Layout::from_size_align(128, 8).unwrap(),

        non_null: NonNull::from(mut_ref1),

        buf_reader: BufReader::new(File::open("/dev/null").unwrap()),
        buf_writer: BufWriter::new(File::create("/tmp/test_all_types_buf_writer").unwrap()),
        cursor: Cursor::new(vec![1, 2, 3, 4]),
    };

    for flags in [
        SizeFlags::empty(),
        SizeFlags::default(),
        SizeFlags::CAPACITY,
        SizeFlags::FOLLOW_REFS,
        SizeFlags::CAPACITY | SizeFlags::FOLLOW_REFS,
    ] {
        let sz = all_types.mem_size(flags);
        assert!(sz > 0, "mem_size with flags {:?} should be > 0", flags);
    }

    // Test with all combinations of depth and flags
    for flags in [
        DbgFlags::default(),
        DbgFlags::CAPACITY,
        DbgFlags::COLOR,
        DbgFlags::HUMANIZE,
        DbgFlags::CAPACITY | DbgFlags::COLOR,
    ] {
        dbg!(flags);
        assert!(
            all_types.mem_dbg(flags).is_ok(),
            "mem_dbg with flags {:?} should succeed",
            flags
        );
        let mut output = String::new();
        assert!(
            all_types.mem_dbg_on(&mut output, flags).is_ok(),
            "mem_dbg_on with flags {:?} should succeed",
            flags
        );
        for depth in 0..5 {
            assert!(
                all_types.mem_dbg_depth(depth, flags).is_ok(),
                "mem_dbg_depth with depth {} and flags {:?} should succeed",
                depth,
                flags
            );
            let mut depth_output = String::new();
            assert!(
                all_types
                    .mem_dbg_depth_on(&mut depth_output, depth, flags)
                    .is_ok(),
                "mem_dbg_depth_on with depth {} and flags {:?} should succeed",
                depth,
                flags
            );
        }
    }
}
