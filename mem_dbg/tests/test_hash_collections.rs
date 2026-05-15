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

// Mirror of the constants in impl_mem_size.rs, which in turn match what the
// stdlib's vendored hashbrown actually uses: SSE2 on x86/x86_64 (16-byte
// groups) and the generic 8-byte path everywhere else (including aarch64+NEON,
// where hashbrown's `Group` is `uint8x8_t`, still 8 bytes). These tests are
// the byte-exact ground truth; test_correctness.rs cross-checks within
// tolerance against the cap allocator.
#[cfg(all(
    any(target_arch = "x86_64", target_arch = "x86"),
    any(target_feature = "sse2", target_env = "msvc"),
))]
const GROUP_WIDTH: usize = 16;
#[cfg(not(all(
    any(target_arch = "x86_64", target_arch = "x86"),
    any(target_feature = "sse2", target_env = "msvc"),
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

fn predicted_set_size<K>(set: &HashSet<K>, flags: SizeFlags, items_size: usize) -> usize {
    let cap = if flags.contains(SizeFlags::CAPACITY) {
        set.capacity()
    } else {
        set.len()
    };
    let buckets = capacity_to_buckets(cap).unwrap_or(usize::MAX);
    core::mem::size_of::<HashSet<K>>()
        + items_size
        + (buckets - set.len()) * core::mem::size_of::<K>()
        + buckets * core::mem::size_of::<u8>()
        + if buckets > 0 { GROUP_WIDTH } else { 0 }
}

fn predicted_map_size<K, V>(map: &HashMap<K, V>, flags: SizeFlags, items_size: usize) -> usize {
    let cap = if flags.contains(SizeFlags::CAPACITY) {
        map.capacity()
    } else {
        map.len()
    };
    let buckets = capacity_to_buckets(cap).unwrap_or(usize::MAX);
    core::mem::size_of::<HashMap<K, V>>()
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
