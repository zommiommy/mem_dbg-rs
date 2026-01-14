use cap::Cap;
use deepsize::*;
use get_size::*;
use mem_dbg::*;
use std::alloc;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

fn main() {
    const N: usize = 100_000_000;

    {
        println!("--- BTreeMap ---");
        let mut m = BTreeMap::new();
        for i in 0..N {
            m.insert(i, i);
        }

        println!("Allocated:    {}", ALLOCATOR.allocated());

        let start = Instant::now();
        let size = m.get_size();
        println!("get_size:     {} {:?} ns", size, start.elapsed().as_nanos());

        let start = Instant::now();
        let size = m.deep_size_of();
        println!("deep_size_of: {} {:?} ns", size, start.elapsed().as_nanos());

        // Commented out as `size-of` crate is currently broken.
        // let start = Instant::now();
        // let size = m.size_of().total_bytes();
        // println!("size_of:      {} {:?} ns", size, start.elapsed().as_nanos());

        let start = Instant::now();
        let size = m.mem_size(SizeFlags::default());
        println!("mem_size:     {} {:?} ns", size, start.elapsed().as_nanos());
    }

    {
        println!("\n--- BTreeMap<String, String> ---");
        // Use a smaller N for String to avoid excessive memory usage/time
        const N_STR: usize = 1_000_000;
        let mut m = BTreeMap::new();
        for i in 0..N_STR {
            m.insert(i.to_string(), i.to_string());
        }

        println!("Allocated:    {}", ALLOCATOR.allocated());

        let start = Instant::now();
        let size = m.get_size();
        println!("get_size:     {} {:?} ns", size, start.elapsed().as_nanos());

        let start = Instant::now();
        let size = m.deep_size_of();
        println!("deep_size_of: {} {:?} ns", size, start.elapsed().as_nanos());

        let start = Instant::now();
        let size = m.mem_size(SizeFlags::default());
        println!("mem_size:     {} {:?} ns", size, start.elapsed().as_nanos());
    }

    {
        println!("\n--- BTreeSet ---");
        let mut m = BTreeSet::new();
        for i in 0..N {
            m.insert(i);
        }

        println!("Allocated:    {}", ALLOCATOR.allocated());

        let start = Instant::now();
        let size = m.get_size();
        println!("get_size:     {} {:?} ns", size, start.elapsed().as_nanos());

        let start = Instant::now();
        let size = m.deep_size_of();
        println!("deep_size_of: {} {:?} ns", size, start.elapsed().as_nanos());

        // Commented out as `size-of` crate is currently broken.
        // let start = Instant::now();
        // let size = m.size_of().total_bytes();
        // println!("size_of:      {} {:?} ns", size, start.elapsed().as_nanos());

        let start = Instant::now();
        let size = m.mem_size(SizeFlags::default());
        println!("mem_size:     {} {:?} ns", size, start.elapsed().as_nanos());
    }
}
