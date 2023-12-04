use cap::Cap;
use deepsize::*;
use get_size::*;
use mem_dbg::SizeFlags;
use mem_dbg::*;
use size_of::*;
use std::alloc;
use std::collections::HashMap;
use std::time::Instant;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

fn main() {
    const N: usize = 100_000_000;
    let mut m = HashMap::with_capacity(N);
    for i in 0..N {
        m.insert(i, i);
    }

    println!("Allocated: {}", ALLOCATOR.allocated());

    let start = Instant::now();
    let size = m.get_size();
    println!("get_size: {} {:?}", size, start.elapsed());

    let start = Instant::now();
    let size = m.deep_size_of();
    println!("deep_size_of: {} {:?}", size, start.elapsed());

    let start = Instant::now();
    let size = m.size_of().total_bytes();
    println!("size_of: {} {:?}", size, start.elapsed());

    let start = Instant::now();
    let size = m.mem_size(SizeFlags::default());
    println!("mem_size: {} {:?}", size, start.elapsed());
}
