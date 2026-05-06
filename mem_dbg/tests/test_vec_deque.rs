#![cfg(feature = "std")]
use mem_dbg::*;
use std::collections::VecDeque;

#[test]
fn test_vec_deque_mem_size() {
    let mut v: VecDeque<u32> = VecDeque::new();
    v.push_back(1);
    v.push_back(2);

    // Default flags: stack frame plus len * size_of::<T>().
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        core::mem::size_of::<VecDeque<u32>>() + v.len() * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_vec_deque_capacity() {
    let mut v: VecDeque<u32> = VecDeque::with_capacity(10);
    v.push_back(1);
    v.push_back(2);

    let size_default = v.mem_size(SizeFlags::default());
    let size_capacity = v.mem_size(SizeFlags::CAPACITY);

    assert_eq!(
        size_default,
        core::mem::size_of::<VecDeque<u32>>() + v.len() * core::mem::size_of::<u32>()
    );
    assert_eq!(
        size_capacity,
        core::mem::size_of::<VecDeque<u32>>() + v.capacity() * core::mem::size_of::<u32>()
    );
    // The CAPACITY-flag delta is exactly the unused capacity in bytes.
    assert_eq!(
        size_capacity - size_default,
        (v.capacity() - v.len()) * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_vec_deque_with_heap_elements() {
    let v: VecDeque<String> = VecDeque::from_iter(["hello".to_string(), "world".to_string()]);
    let inner_size: usize = v
        .iter()
        .map(|s| <String as MemSize>::mem_size(s, SizeFlags::default()))
        .sum();
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        core::mem::size_of::<VecDeque<String>>() + inner_size
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
