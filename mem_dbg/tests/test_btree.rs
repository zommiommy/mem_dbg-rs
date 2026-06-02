#![cfg(feature = "std")]
use mem_dbg::*;
use std::collections::{BTreeMap, BTreeSet};

/// Compute the leaf node size for a BTree with the standard library's B=6,
/// mirroring the formula in `estimate_btree_size`.
fn btree_leaf_size<K, V>() -> usize {
    const CAPACITY: usize = 2 * 6 - 1; // B=6, so 11

    let align_up = |size: usize, align: usize| -> usize { (size + align - 1) & !(align - 1) };

    let header_size = 2 * core::mem::size_of::<usize>();
    let mut leaf_size = header_size;
    leaf_size = align_up(leaf_size, core::mem::align_of::<K>());
    leaf_size += core::mem::size_of::<K>() * CAPACITY;
    leaf_size = align_up(leaf_size, core::mem::align_of::<V>());
    leaf_size += core::mem::size_of::<V>() * CAPACITY;
    leaf_size
}

#[test]
fn test_btree_map() {
    let mut map = BTreeMap::new();
    map.insert(1u8, 2u8);
    map.insert(3u8, 4u8);

    let size = map.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeMap<u8, u8>>() + btree_leaf_size::<u8, u8>();
    assert_eq!(size, expected);
}

#[test]
fn test_btree_set() {
    let mut set = BTreeSet::new();
    set.insert(1u8);
    set.insert(2u8);

    let size = set.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeSet<u8>>() + btree_leaf_size::<u8, ()>();
    assert_eq!(size, expected);
}

#[test]
fn test_btree_map_recursive() {
    let mut map = BTreeMap::new();
    map.insert(1u8, vec![1u8, 2u8]);

    // Leaf node size + Vec heap (2 bytes for the 2 u8 elements)
    let size = map.mem_size(SizeFlags::default());
    let expected =
        std::mem::size_of::<BTreeMap<u8, Vec<u8>>>() + btree_leaf_size::<u8, Vec<u8>>() + 2;
    assert_eq!(size, expected);
}

#[test]
fn test_btree_map_string_keys() {
    // Non-flat key, flat value: heap comes only from the keys.
    let map: BTreeMap<String, u8> = (0..3).map(|x| (x.to_string(), x as u8)).collect();
    let item_heap: usize = map
        .keys()
        .map(|k| {
            <String as MemSize>::mem_size(k, SizeFlags::default()) - core::mem::size_of::<String>()
        })
        .sum();
    let size = map.mem_size(SizeFlags::default());
    let expected =
        std::mem::size_of::<BTreeMap<String, u8>>() + btree_leaf_size::<String, u8>() + item_heap;
    assert_eq!(size, expected);
}

#[test]
fn test_btree_map_string_keys_and_values() {
    // Non-flat key and value: heap comes from both, in a single pass over entries.
    let map: BTreeMap<String, String> = (0..3).map(|x| (x.to_string(), x.to_string())).collect();
    let item_heap: usize = map
        .iter()
        .map(|(k, v)| {
            (<String as MemSize>::mem_size(k, SizeFlags::default())
                - core::mem::size_of::<String>())
                + (<String as MemSize>::mem_size(v, SizeFlags::default())
                    - core::mem::size_of::<String>())
        })
        .sum();
    let size = map.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeMap<String, String>>()
        + btree_leaf_size::<String, String>()
        + item_heap;
    assert_eq!(size, expected);
}
