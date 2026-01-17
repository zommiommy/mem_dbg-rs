use criterion::{Criterion, criterion_group, criterion_main};
use deepsize::DeepSizeOf;
use get_size::GetSize;
use mem_dbg::*;
use std::{collections::HashMap, hint::black_box};

#[derive(MemSize, DeepSizeOf, GetSize, Clone)]
struct InnerStruct {
    x: Vec<u32>,
    y: Option<Box<[u8]>>,
    z: String,
}

#[derive(MemSize, DeepSizeOf, GetSize, Clone)]
struct TestStruct {
    a: Vec<u8>,
    b: HashMap<String, InnerStruct>,
    c: Option<Box<[u64]>>,
    d: String,
}

fn bench_mem_size(c: &mut Criterion) {
    let mut map = HashMap::new();
    for i in 0..1000 {
        let inner = InnerStruct {
            x: vec![i; 10_000],
            y: if i % 2 == 0 {
                Some(vec![0; 50].into_boxed_slice())
            } else {
                None
            },
            z: format!("val_{}", i),
        };
        map.insert(format!("key_{}", i), inner);
    }

    let data = TestStruct {
        a: vec![0; 100_000],
        b: map,
        c: Some(vec![123; 10_000].into_boxed_slice()),
        d: "Root".to_string(),
    };

    let mut group = c.benchmark_group("Size Measurement");

    group.bench_function("mem_dbg", |b| {
        b.iter(|| black_box(&data).mem_size(SizeFlags::default()))
    });

    group.bench_function("deepsize", |b| b.iter(|| black_box(&data).deep_size_of()));

    group.bench_function("get-size", |b| b.iter(|| black_box(&data).get_size()));

    group.finish();
}

criterion_group!(benches, bench_mem_size);
criterion_main!(benches);
