#![cfg(feature = "std")]
use mem_dbg::*;
use std::collections::LinkedList;

#[test]
fn test_linked_list_empty() {
    let l: LinkedList<u32> = LinkedList::new();
    assert_eq!(
        l.mem_size(SizeFlags::default()),
        core::mem::size_of::<LinkedList<u32>>()
    );
}

#[test]
fn test_linked_list_flat_elements() {
    let mut l: LinkedList<u32> = LinkedList::new();
    l.push_back(1);
    l.push_back(2);
    l.push_back(3);

    let expected = core::mem::size_of::<LinkedList<u32>>()
        + l.len() * core::mem::size_of::<LinkedListNode<u32>>();
    assert_eq!(l.mem_size(SizeFlags::default()), expected);
}

#[test]
fn test_linked_list_capacity_matches_default() {
    // LinkedList allocates per node and has no reserved spare capacity, so
    // `SizeFlags::CAPACITY` must return the same value as the default flags.
    let mut l: LinkedList<u32> = LinkedList::new();
    l.push_back(10);
    l.push_back(20);
    assert_eq!(
        l.mem_size(SizeFlags::default()),
        l.mem_size(SizeFlags::CAPACITY)
    );
}

#[test]
fn test_linked_list_with_heap_elements() {
    let mut l: LinkedList<String> = LinkedList::new();
    l.push_back("hello".to_string());
    l.push_back("world".to_string());

    let per_node_overhead =
        core::mem::size_of::<LinkedListNode<String>>() - core::mem::size_of::<String>();
    let inner: usize = l
        .iter()
        .map(|s| <String as MemSize>::mem_size(s, SizeFlags::default()) + per_node_overhead)
        .sum();
    assert_eq!(
        l.mem_size(SizeFlags::default()),
        core::mem::size_of::<LinkedList<String>>() + inner
    );
}

#[test]
fn test_linked_list_aligned_element_node_size() {
    // Sanity check that the mirror struct correctly accounts for elements
    // whose alignment exceeds pointer alignment.
    #[repr(align(32))]
    #[derive(Clone, Copy)]
    #[allow(dead_code)]
    struct Aligned32(u8);

    impl FlatType for Aligned32 {
        type Flat = True;
    }
    impl MemSize for Aligned32 {
        fn mem_size_rec(
            &self,
            _flags: SizeFlags,
            _refs: &mut mem_dbg::HashMap<usize, usize>,
        ) -> usize {
            core::mem::size_of::<Self>()
        }
    }

    let mut l: LinkedList<Aligned32> = LinkedList::new();
    l.push_back(Aligned32(1));
    l.push_back(Aligned32(2));

    let expected = core::mem::size_of::<LinkedList<Aligned32>>()
        + l.len() * core::mem::size_of::<LinkedListNode<Aligned32>>();
    assert_eq!(l.mem_size(SizeFlags::default()), expected);
}

#[test]
fn test_linked_list_mem_dbg() {
    let mut l: LinkedList<u32> = LinkedList::new();
    l.push_back(1);
    l.push_back(2);
    l.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
fn test_linked_list_mem_dbg_depth() {
    let mut l: LinkedList<u32> = LinkedList::new();
    l.push_back(1);

    for depth in 0..3 {
        let result = l.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
