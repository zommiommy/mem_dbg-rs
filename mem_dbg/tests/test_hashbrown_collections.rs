#![cfg(feature = "hashbrown")]

//! Size assertions for `hashbrown::HashSet<T>` and `hashbrown::HashMap<K, V>`.
//!
//! Mirror of `test_hash_collections.rs` for the optional `hashbrown` feature.
//! The `MemSize` implementation rides on the same Swiss-table layout math
//! (`capacity_to_buckets` + `GROUP_WIDTH`); here we replicate the formula and
//! assert agreement, exactly like the std variant.
//!
//! End-to-end agreement against the runtime allocator is exercised by
//! `test_correctness.rs` for the std collections; the hashbrown impls share
//! the same arithmetic so a separate cap-based test isn't needed.

use hashbrown::{HashMap, HashSet};
use mem_dbg::*;

// Mirror of the constants in impl_mem_size.rs. GROUP_WIDTH is 16 for SSE2/NEON
// targets and 8 elsewhere.
#[cfg(any(
    target_feature = "sse2",
    all(target_arch = "aarch64", target_feature = "neon"),
))]
const GROUP_WIDTH: usize = 16;
#[cfg(not(any(
    target_feature = "sse2",
    all(target_arch = "aarch64", target_feature = "neon"),
)))]
const GROUP_WIDTH: usize = 8;

fn capacity_to_buckets(cap: usize) -> Option<usize> {
    if cap == 0 {
        return Some(0);
    }
    if cap < 8 {
        return Some(if cap < 4 { 4 } else { 8 });
    }
    let adjusted_cap = cap.checked_mul(8)? / 7;
    Some(adjusted_cap.next_power_of_two())
}

fn predicted_set_size<K, S>(set: &HashSet<K, S>, flags: SizeFlags, items_size: usize) -> usize {
    let cap = if flags.contains(SizeFlags::CAPACITY) {
        set.capacity()
    } else {
        set.len()
    };
    let buckets = capacity_to_buckets(cap).unwrap_or(usize::MAX);
    core::mem::size_of::<HashSet<K, S>>()
        + items_size
        + (buckets - set.len()) * core::mem::size_of::<K>()
        + buckets * core::mem::size_of::<u8>()
        + if buckets > 0 { GROUP_WIDTH } else { 0 }
}

fn predicted_map_size<K, V, S>(
    map: &HashMap<K, V, S>,
    flags: SizeFlags,
    items_size: usize,
) -> usize {
    let cap = if flags.contains(SizeFlags::CAPACITY) {
        map.capacity()
    } else {
        map.len()
    };
    let buckets = capacity_to_buckets(cap).unwrap_or(usize::MAX);
    core::mem::size_of::<HashMap<K, V, S>>()
        + items_size
        + (buckets - map.len()) * (core::mem::size_of::<K>() + core::mem::size_of::<V>())
        + buckets * core::mem::size_of::<u8>()
        + if buckets > 0 { GROUP_WIDTH } else { 0 }
}

#[test]
fn test_empty_hashset() {
    let set: HashSet<u32> = HashSet::new();
    assert_eq!(
        set.mem_size(SizeFlags::default()),
        core::mem::size_of::<HashSet<u32>>()
    );
}

#[test]
fn test_empty_hashmap() {
    let map: HashMap<u32, u32> = HashMap::new();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        core::mem::size_of::<HashMap<u32, u32>>()
    );
}

#[test]
fn test_hashset_flat_keys() {
    let set: HashSet<u32> = (0..32).collect();
    let item_size = core::mem::size_of::<u32>() * set.len();
    assert_eq!(
        set.mem_size(SizeFlags::default()),
        predicted_set_size(&set, SizeFlags::default(), item_size)
    );
    assert_eq!(
        set.mem_size(SizeFlags::CAPACITY),
        predicted_set_size(&set, SizeFlags::CAPACITY, item_size)
    );
}

#[test]
fn test_hashset_string_keys() {
    let set: HashSet<alloc::string::String> = (0..16).map(|x| x.to_string()).collect();
    let item_size: usize = set
        .iter()
        .map(|s| <alloc::string::String as MemSize>::mem_size(s, SizeFlags::default()))
        .sum();
    assert_eq!(
        set.mem_size(SizeFlags::default()),
        predicted_set_size(&set, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_hashmap_flat_keys_and_values() {
    let map: HashMap<u32, u64> = (0..32).map(|x| (x, x as u64)).collect();
    let item_size = (core::mem::size_of::<u32>() + core::mem::size_of::<u64>()) * map.len();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size(&map, SizeFlags::default(), item_size)
    );
    assert_eq!(
        map.mem_size(SizeFlags::CAPACITY),
        predicted_map_size(&map, SizeFlags::CAPACITY, item_size)
    );
}

#[test]
fn test_hashmap_string_keys() {
    let map: HashMap<alloc::string::String, u32> =
        (0..16).map(|x| (x.to_string(), x as u32)).collect();
    let item_size: usize = map
        .iter()
        .map(|(k, v)| {
            <alloc::string::String as MemSize>::mem_size(k, SizeFlags::default())
                + <u32 as MemSize>::mem_size(v, SizeFlags::default())
                - core::mem::size_of::<u32>()
        })
        .sum::<usize>()
        + core::mem::size_of::<u32>() * map.len();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size(&map, SizeFlags::default(), item_size)
    );
}

extern crate alloc;
