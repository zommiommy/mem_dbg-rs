//! Micro-benchmarks for `MemSize::mem_size` on the standard-library containers.
//!
//! Flat-element cases such as `Vec<u64>` exercise the constant-time size path.
//! Non-flat cases such as `Vec<String>` exercise the per-element iteration
//! path. Compare revisions with `cargo bench -- --save-baseline <name>` and
//! `--baseline <name>` to check for regressions.

// Miri cannot run this harness: criterion's startup spawns a process to probe
// for gnuplot, which Miri's isolation forbids, and interpreting a benchmark is
// pointless anyway. Compile the harness only outside Miri and give Miri a bare
// entry point (a `harness = false` target still needs a `main`).
#[cfg(miri)]
fn main() {}

#[cfg(not(miri))]
mod harness {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};

    use criterion::Criterion;
    use mem_dbg::{MemSize, SizeFlags};

    const N: usize = 100_000;

    /// Times `mem_size` on a referenced value. The caller passes the reference
    /// so the same macro works for both owned collections (`&v`) and slices.
    macro_rules! bench {
        ($group:expr, $name:expr, $value:expr) => {
            bench!($group, $name, $value, SizeFlags::default());
        };
        ($group:expr, $name:expr, $value:expr, $flags:expr) => {
            $group.bench_function($name, |b| {
                b.iter(|| std::hint::black_box($value).mem_size($flags));
            });
        };
    }

    fn flat_vec() -> Vec<u64> {
        (0..N as u64).collect()
    }

    fn non_flat_vec() -> Vec<String> {
        (0..N as u64).map(|i| i.to_string()).collect()
    }

    pub fn flat_paths(c: &mut Criterion) {
        let mut group = c.benchmark_group("flat");

        let v = flat_vec();
        let slice: &[u64] = &v;
        let deque: VecDeque<u64> = (0..N as u64).collect();
        let set: HashSet<u64> = (0..N as u64).collect();
        let map: HashMap<u64, u64> = (0..N as u64).map(|i| (i, i)).collect();
        let bset: BTreeSet<u64> = (0..N as u64).collect();
        let bmap: BTreeMap<u64, u64> = (0..N as u64).map(|i| (i, i)).collect();
        let list: LinkedList<u64> = (0..N as u64).collect();

        bench!(group, "vec_u64", &v);
        bench!(group, "vec_u64_capacity", &v, SizeFlags::CAPACITY);
        bench!(group, "slice_u64", slice);
        bench!(group, "vecdeque_u64", &deque);
        bench!(group, "hashset_u64", &set);
        bench!(group, "hashmap_u64", &map);
        bench!(group, "btreeset_u64", &bset);
        bench!(group, "btreemap_u64", &bmap);
        bench!(group, "linkedlist_u64", &list);

        group.finish();
    }

    pub fn non_flat_paths(c: &mut Criterion) {
        let mut group = c.benchmark_group("non_flat");

        let v = non_flat_vec();
        let deque: VecDeque<String> = non_flat_vec().into_iter().collect();
        let map: HashMap<u64, String> = (0..N as u64).map(|i| (i, i.to_string())).collect();
        let bmap: BTreeMap<u64, String> = (0..N as u64).map(|i| (i, i.to_string())).collect();
        let smap: HashMap<String, String> = (0..N as u64)
            .map(|i| (i.to_string(), i.to_string()))
            .collect();
        let sbmap: BTreeMap<String, String> = (0..N as u64)
            .map(|i| (i.to_string(), i.to_string()))
            .collect();
        let list: LinkedList<String> = non_flat_vec().into_iter().collect();

        bench!(group, "vec_string", &v);
        bench!(group, "vecdeque_string", &deque);
        bench!(group, "hashmap_u64_string", &map);
        bench!(group, "btreemap_u64_string", &bmap);
        bench!(group, "hashmap_string_string", &smap);
        bench!(group, "btreemap_string_string", &sbmap);
        bench!(group, "linkedlist_string", &list);

        group.finish();
    }
}

#[cfg(not(miri))]
criterion::criterion_group!(benches, harness::flat_paths, harness::non_flat_paths);
#[cfg(not(miri))]
criterion::criterion_main!(benches);
