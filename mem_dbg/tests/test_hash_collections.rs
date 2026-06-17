#![cfg(feature = "std")]

//! Size assertions for `HashSet<T>` and `HashMap<K, V>`.
//!
//! Predicting the *actual* heap occupancy of these collections is hard
//! (hashbrown's `HashMap` uses Swiss Tables and a load-factor-driven growth
//! policy). The `MemSize` implementation reports an estimate based on the
//! capacity-to-buckets formula plus the GROUP_WIDTH speedup constant; here we
//! replicate that formula and assert the impl matches it exactly. End-to-end
//! agreement against the runtime allocator is exercised by
//! `test_correctness.rs` with a configurable tolerance.

use mem_dbg::*;
use std::collections::{HashMap, HashSet};

// Mirror of the constants in impl_mem_size.rs, which in turn match the
// stdlib's vendored hashbrown 0.16 RawTable layout.
#[cfg(all(
    any(target_arch = "x86_64", target_arch = "x86"),
    target_feature = "sse2",
    not(miri),
))]
const GROUP_WIDTH: usize = 16;
#[cfg(all(
    not(all(
        any(target_arch = "x86_64", target_arch = "x86"),
        target_feature = "sse2",
        not(miri),
    )),
    any(
        target_pointer_width = "64",
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "wasm32",
    ),
))]
const GROUP_WIDTH: usize = 8;
#[cfg(all(
    not(all(
        any(target_arch = "x86_64", target_arch = "x86"),
        target_feature = "sse2",
        not(miri),
    )),
    not(any(
        target_pointer_width = "64",
        target_arch = "aarch64",
        target_arch = "x86_64",
        target_arch = "wasm32",
    )),
))]
const GROUP_WIDTH: usize = 4;

fn capacity_to_buckets<T>(cap: usize) -> Option<usize> {
    if cap == 0 {
        return Some(0);
    }
    if cap < 15 {
        let min_cap = match (GROUP_WIDTH, core::mem::size_of::<T>()) {
            (16, 0..=1) => 14,
            (16, 2..=3) => 7,
            (8, 0..=1) => 7,
            _ => 3,
        };
        let cap = core::cmp::max(cap, min_cap);
        return Some(if cap < 4 {
            4
        } else if cap < 8 {
            8
        } else {
            16
        });
    }
    let adjusted_cap = cap.checked_mul(8)? / 7;
    Some(adjusted_cap.next_power_of_two())
}

fn hash_table_allocation_size<T>(capacity: usize) -> usize {
    let buckets = capacity_to_buckets::<T>(capacity).unwrap();
    if buckets == 0 {
        return 0;
    }

    let ctrl_align = core::cmp::max(core::mem::align_of::<T>(), GROUP_WIDTH);
    let bucket_bytes = core::mem::size_of::<T>() * buckets;
    let ctrl_offset = (bucket_bytes + ctrl_align - 1) & !(ctrl_align - 1);
    ctrl_offset + buckets + GROUP_WIDTH
}

fn predicted_set_size<K>(set: &HashSet<K>, flags: SizeFlags, items_size: usize) -> usize {
    let cap = if flags.contains(SizeFlags::CAPACITY) {
        set.capacity()
    } else {
        set.len()
    };
    let heap_extras = items_size - set.len() * core::mem::size_of::<K>();
    core::mem::size_of::<HashSet<K>>() + hash_table_allocation_size::<K>(cap) + heap_extras
}

fn predicted_map_size<K, V>(map: &HashMap<K, V>, flags: SizeFlags, items_size: usize) -> usize {
    let cap = if flags.contains(SizeFlags::CAPACITY) {
        map.capacity()
    } else {
        map.len()
    };
    let inline_size = map.len() * (core::mem::size_of::<K>() + core::mem::size_of::<V>());
    let heap_extras = items_size - inline_size;
    core::mem::size_of::<HashMap<K, V>>() + hash_table_allocation_size::<(K, V)>(cap) + heap_extras
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
        predicted_set_size::<u32>(&set, SizeFlags::default(), item_size)
    );
    assert_eq!(
        set.mem_size(SizeFlags::CAPACITY),
        predicted_set_size::<u32>(&set, SizeFlags::CAPACITY, item_size)
    );
}

#[test]
fn test_hashset_string_keys() {
    let set: HashSet<String> = (0..16).map(|x| x.to_string()).collect();
    let item_size: usize = set
        .iter()
        .map(|s| <String as MemSize>::mem_size(s, SizeFlags::default()))
        .sum();
    assert_eq!(
        set.mem_size(SizeFlags::default()),
        predicted_set_size::<String>(&set, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_hashmap_flat_keys_and_values() {
    let map: HashMap<u32, u64> = (0..32).map(|x| (x, x as u64)).collect();
    let item_size = (core::mem::size_of::<u32>() + core::mem::size_of::<u64>()) * map.len();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size::<u32, u64>(&map, SizeFlags::default(), item_size)
    );
    assert_eq!(
        map.mem_size(SizeFlags::CAPACITY),
        predicted_map_size::<u32, u64>(&map, SizeFlags::CAPACITY, item_size)
    );
}

#[test]
fn test_hashmap_string_keys() {
    let map: HashMap<String, u32> = (0..16).map(|x| (x.to_string(), x as u32)).collect();
    let item_size: usize = map
        .iter()
        .map(|(k, v)| {
            <String as MemSize>::mem_size(k, SizeFlags::default())
                + <u32 as MemSize>::mem_size(v, SizeFlags::default())
                - core::mem::size_of::<u32>()
        })
        .sum::<usize>()
        + core::mem::size_of::<u32>() * map.len();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size::<String, u32>(&map, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_hashmap_flat_keys_string_values() {
    // Flat key, non-flat value: exercises the mixed single-pass path where the
    // key contributes only its inline slot and the value also contributes heap.
    let map: HashMap<u32, String> = (0..16).map(|x| (x, x.to_string())).collect();
    let item_size: usize = map
        .iter()
        .map(|(k, v)| {
            <u32 as MemSize>::mem_size(k, SizeFlags::default())
                + <String as MemSize>::mem_size(v, SizeFlags::default())
        })
        .sum();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size::<u32, String>(&map, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_hashmap_string_keys_and_values() {
    // Non-flat key and value: the fully recursive single-pass path where both
    // key and value contribute heap beyond their inline slots.
    let map: HashMap<String, String> = (0..16).map(|x| (x.to_string(), x.to_string())).collect();
    let item_size: usize = map
        .iter()
        .map(|(k, v)| {
            <String as MemSize>::mem_size(k, SizeFlags::default())
                + <String as MemSize>::mem_size(v, SizeFlags::default())
        })
        .sum();
    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size::<String, String>(&map, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_hashmap_uses_tuple_bucket_layout() {
    assert!(
        core::mem::size_of::<(u8, u64)>()
            > core::mem::size_of::<u8>() + core::mem::size_of::<u64>()
    );

    let map: HashMap<u8, u64> = [(1, 2)].into_iter().collect();
    let item_size = core::mem::size_of::<u8>() + core::mem::size_of::<u64>();

    assert_eq!(
        map.mem_size(SizeFlags::default()),
        predicted_map_size::<u8, u64>(&map, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_hash_table_small_capacity_rules_match_hashbrown_0_16() {
    let buckets = capacity_to_buckets::<u8>(1).unwrap();
    if GROUP_WIDTH == 16 {
        assert_eq!(buckets, 16);
    } else if GROUP_WIDTH == 8 {
        assert_eq!(buckets, 8);
    } else {
        assert_eq!(buckets, 4);
    }
}

#[test]
fn test_hash_table_control_offset_is_aligned() {
    let buckets = capacity_to_buckets::<[u8; 3]>(7).unwrap();
    let bucket_bytes = core::mem::size_of::<[u8; 3]>() * buckets;
    let ctrl_align = core::cmp::max(core::mem::align_of::<[u8; 3]>(), GROUP_WIDTH);
    let ctrl_offset = (bucket_bytes + ctrl_align - 1) & !(ctrl_align - 1);

    assert!(ctrl_offset > bucket_bytes);

    let set: HashSet<[u8; 3]> = (0..7).map(|x| [x, 0, 0]).collect();
    let item_size = core::mem::size_of::<[u8; 3]>() * set.len();
    assert_eq!(
        set.mem_size(SizeFlags::default()),
        predicted_set_size::<[u8; 3]>(&set, SizeFlags::default(), item_size)
    );
}

#[test]
fn test_zst_hash_table_still_allocates_control_bytes() {
    let table_bytes = hash_table_allocation_size::<()>(1);
    assert!(table_bytes > 0);

    let set: HashSet<()> = [()].into_iter().collect();
    assert_eq!(
        set.mem_size(SizeFlags::default()),
        predicted_set_size::<()>(&set, SizeFlags::default(), 0)
    );
}
