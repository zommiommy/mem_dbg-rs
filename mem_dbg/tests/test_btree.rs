#![cfg(feature = "std")]
use mem_dbg::*;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn test_btree_map() {
    let mut map = BTreeMap::new();
    map.insert(1u8, 2u8);
    map.insert(3u8, 4u8);

    // Size should be:
    // size_of::<BTreeMap>() +
    // Node size (38 bytes for 2 elements in a leaf with B=6, 16b header, aligned)

    let size = map.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeMap<u8, u8>>() + 38;
    assert_eq!(size, expected);
}

#[test]
fn test_btree_set() {
    let mut set = BTreeSet::new();
    set.insert(1u8);
    set.insert(2u8);

    let size = set.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeSet<u8>>() + 27;
    assert_eq!(size, expected);
}

#[test]
fn test_btree_map_recursive() {
    let mut map = BTreeMap::new();
    map.insert(1u8, vec![1u8, 2u8]);

    // size_of::<BTreeMap>
    // + Leaf Node size (296 bytes)
    // + Vec heap overhead (2 bytes)

    let size = map.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeMap<u8, Vec<u8>>>() + 298;
    assert_eq!(size, expected);
}
