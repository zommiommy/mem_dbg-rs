#![cfg(feature = "dashmap")]

//! Size assertions for `dashmap::DashMap<K, V, S>`. DashMap is internally
//! `Box<[CachePadded<RwLock<RawTable<(K, SharedValue<V>)>>>]>` with `SharedValue<V>`
//! being `#[repr(transparent)]` over `V`. The impl walks each shard under a read
//! lock and applies the same Swiss-Table overhead formula used for std maps;
//! this test replicates that formula and checks exact agreement.

use dashmap::DashMap;
use mem_dbg::*;

// Mirror of constants in impl_mem_size.rs: SSE2 on x86/x86_64 only, generic
// 8-byte path everywhere else (including aarch64+NEON).
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

fn predicted_dashmap_size<K, V, S, FK, FV>(
    map: &DashMap<K, V, S>,
    flags: SizeFlags,
    mut key_bytes: FK,
    mut val_bytes: FV,
) -> usize
where
    K: Eq + core::hash::Hash,
    S: core::hash::BuildHasher + Clone,
    FK: FnMut(&K) -> usize,
    FV: FnMut(&V) -> usize,
{
    let mut total = core::mem::size_of::<DashMap<K, V, S>>();
    for shard in map.shards() {
        total += core::mem::size_of_val(shard);
        let guard = shard.read();
        let cap = if flags.contains(SizeFlags::CAPACITY) {
            guard.capacity()
        } else {
            guard.len()
        };
        let buckets = capacity_to_buckets(cap).unwrap_or(usize::MAX);
        let mut entries = 0;
        for bucket in unsafe { guard.iter() } {
            let (k, v) = unsafe { bucket.as_ref() };
            entries += key_bytes(k) + val_bytes(v.get());
        }
        total += entries
            + (buckets - guard.len()) * (core::mem::size_of::<K>() + core::mem::size_of::<V>())
            + buckets * core::mem::size_of::<u8>()
            + if buckets > 0 { GROUP_WIDTH } else { 0 };
    }
    total
}

#[test]
fn flat_flat_empty() {
    let m: DashMap<u64, u64> = DashMap::new();
    let pred = predicted_dashmap_size(
        &m,
        SizeFlags::default(),
        |_| core::mem::size_of::<u64>(),
        |_| core::mem::size_of::<u64>(),
    );
    assert_eq!(m.mem_size(SizeFlags::default()), pred);
}

#[test]
fn flat_flat_populated() {
    let m: DashMap<u64, u64> = DashMap::new();
    for i in 0..100 {
        m.insert(i, i * 2);
    }
    for flags in [SizeFlags::default(), SizeFlags::CAPACITY] {
        let pred = predicted_dashmap_size(
            &m,
            flags,
            |_| core::mem::size_of::<u64>(),
            |_| core::mem::size_of::<u64>(),
        );
        assert_eq!(m.mem_size(flags), pred, "flags={:?}", flags);
    }
}

#[test]
fn flat_nonflat_values() {
    let m: DashMap<u64, String> = DashMap::new();
    for i in 0..32 {
        m.insert(i, format!("value-{i}-with-some-content"));
    }
    for flags in [SizeFlags::default(), SizeFlags::CAPACITY] {
        let pred = predicted_dashmap_size(
            &m,
            flags,
            |_| core::mem::size_of::<u64>(),
            |s: &String| {
                core::mem::size_of::<String>()
                    + if flags.contains(SizeFlags::CAPACITY) {
                        s.capacity()
                    } else {
                        s.len()
                    }
            },
        );
        assert_eq!(m.mem_size(flags), pred, "flags={:?}", flags);
    }
}

#[test]
fn nonflat_flat_keys() {
    let m: DashMap<String, u32> = DashMap::new();
    for i in 0..32 {
        m.insert(format!("key-{i}-padding"), i);
    }
    for flags in [SizeFlags::default(), SizeFlags::CAPACITY] {
        let pred = predicted_dashmap_size(
            &m,
            flags,
            |s: &String| {
                core::mem::size_of::<String>()
                    + if flags.contains(SizeFlags::CAPACITY) {
                        s.capacity()
                    } else {
                        s.len()
                    }
            },
            |_| core::mem::size_of::<u32>(),
        );
        assert_eq!(m.mem_size(flags), pred, "flags={:?}", flags);
    }
}

#[test]
fn nonflat_nonflat() {
    let m: DashMap<String, Vec<u32>> = DashMap::new();
    for i in 0..16 {
        m.insert(format!("k-{i}"), (0..i).collect());
    }
    for flags in [SizeFlags::default(), SizeFlags::CAPACITY] {
        let pred = predicted_dashmap_size(
            &m,
            flags,
            |s: &String| {
                core::mem::size_of::<String>()
                    + if flags.contains(SizeFlags::CAPACITY) {
                        s.capacity()
                    } else {
                        s.len()
                    }
            },
            |v: &Vec<u32>| {
                core::mem::size_of::<Vec<u32>>()
                    + if flags.contains(SizeFlags::CAPACITY) {
                        v.capacity()
                    } else {
                        v.len()
                    } * core::mem::size_of::<u32>()
            },
        );
        assert_eq!(m.mem_size(flags), pred, "flags={:?}", flags);
    }
}
