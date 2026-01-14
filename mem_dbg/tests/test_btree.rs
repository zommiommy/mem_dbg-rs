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
    // 2 * size_of::<u8>() (keys) +
    // 2 * size_of::<u8>() (values)
    // = size_of::<BTreeMap>() + 4 bytes.
    // This implementation ignores internal node overhead.

    let size = map.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeMap<u8, u8>>() + 4;
    assert_eq!(size, expected);
}

#[test]
fn test_btree_set() {
    let mut set = BTreeSet::new();
    set.insert(1u8);
    set.insert(2u8);

    let size = set.mem_size(SizeFlags::default());
    let expected = std::mem::size_of::<BTreeSet<u8>>() + 2;
    assert_eq!(size, expected);
}

#[test]
fn test_btree_map_recursive() {
    let mut map = BTreeMap::new();
    map.insert(1u8, vec![1u8, 2u8]);

    // size_of::<BTreeMap>
    // + 1 * u8
    // + 1 * Vec (size_of::<Vec> + 2 * u8)

    let size = map.mem_size(SizeFlags::default());
    let expected =
        std::mem::size_of::<BTreeMap<u8, Vec<u8>>>() + 1 + std::mem::size_of::<Vec<u8>>() + 2;
    assert_eq!(size, expected);
}
