#![cfg(feature = "std")]
use mem_dbg::*;
use std::collections::VecDeque;

#[test]
fn test_vec_deque_mem_size() {
    let mut v: VecDeque<u32> = VecDeque::new();
    v.push_back(1);
    v.push_back(2);

    // Size check
    let size = v.mem_size(SizeFlags::default());
    // struct size + 2 * u32
    assert!(size >= core::mem::size_of::<VecDeque<u32>>());
    assert!(size >= core::mem::size_of::<VecDeque<u32>>() + 2 * core::mem::size_of::<u32>());
}

#[test]
fn test_vec_deque_capacity() {
    let mut v: VecDeque<u32> = VecDeque::with_capacity(10);
    v.push_back(1);
    v.push_back(2);

    let size_default = v.mem_size(SizeFlags::default());
    let size_capacity = v.mem_size(SizeFlags::CAPACITY);

    // Capacity should be included
    assert!(size_capacity >= size_default);

    assert_eq!(
        size_capacity - size_default,
        (v.capacity() - v.len()) * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_vec_deque_mem_dbg() {
    let mut v: VecDeque<u32> = VecDeque::new();
    v.push_back(100);
    v.mem_dbg(DbgFlags::default()).unwrap();

    for depth in 0..3 {
        let result = v.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
