#![cfg(feature = "std")]

//! Size assertions for `BinaryHeap<T>`.
//!
//! The implementation follows the same Vec-backed element and capacity
//! accounting used by `Vec<T>`.

use mem_dbg::{DbgFlags, MemDbg, MemSize, SizeFlags};
use std::collections::BinaryHeap;

#[test]
fn test_binary_heap_mem_size() {
    let mut heap = BinaryHeap::new();
    heap.push(1_u32);
    heap.push(2);

    assert_eq!(
        heap.mem_size(SizeFlags::default()),
        core::mem::size_of::<BinaryHeap<u32>>() + heap.len() * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_binary_heap_capacity() {
    let mut heap = BinaryHeap::with_capacity(10);
    heap.push(1_u32);
    heap.push(2);

    let size_default = heap.mem_size(SizeFlags::default());
    let size_capacity = heap.mem_size(SizeFlags::CAPACITY);

    assert_eq!(
        size_default,
        core::mem::size_of::<BinaryHeap<u32>>() + heap.len() * core::mem::size_of::<u32>()
    );
    assert_eq!(
        size_capacity,
        core::mem::size_of::<BinaryHeap<u32>>() + heap.capacity() * core::mem::size_of::<u32>()
    );
    assert_eq!(
        size_capacity - size_default,
        (heap.capacity() - heap.len()) * core::mem::size_of::<u32>()
    );
}

#[test]
fn test_binary_heap_with_heap_elements() {
    let heap = BinaryHeap::from(["hello".to_string(), "world".to_string()]);
    let inner_size: usize = heap
        .iter()
        .map(|s| <String as MemSize>::mem_size(s, SizeFlags::default()))
        .sum();

    assert_eq!(
        heap.mem_size(SizeFlags::default()),
        core::mem::size_of::<BinaryHeap<String>>() + inner_size
    );
}

#[test]
fn test_binary_heap_with_heap_elements_capacity() {
    let mut heap = BinaryHeap::with_capacity(8);
    heap.push("hello".to_string());
    heap.push("world".to_string());

    let inner_size: usize = heap
        .iter()
        .map(|s| <String as MemSize>::mem_size(s, SizeFlags::default()))
        .sum();

    assert_eq!(
        heap.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<BinaryHeap<String>>()
            + inner_size
            + (heap.capacity() - heap.len()) * core::mem::size_of::<String>()
    );
}

#[test]
fn test_binary_heap_mem_dbg() {
    let mut heap = BinaryHeap::new();
    heap.push(100_u32);
    let mut output = String::new();

    for depth in 0..3 {
        output.clear();
        assert!(
            heap.mem_dbg_depth_on(&mut output, depth, DbgFlags::default())
                .is_ok()
        );
    }
}
