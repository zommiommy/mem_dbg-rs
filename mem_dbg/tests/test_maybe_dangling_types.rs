#![cfg(all(feature = "maybe_dangling", feature = "std", feature = "derive"))]

//! `MaybeDangling<T>` is a `#[repr(transparent)]` wrapper over `T`, so its
//! `MemSize`/`MemDbg` behaviour must match the wrapped `T` exactly.

use maybe_dangling::MaybeDangling;
use mem_dbg::*;

#[test]
fn test_maybe_dangling_flat_payload() {
    let md = MaybeDangling::new(42u32);
    // Transparent over `T`: same size as `T`, no heap delta.
    assert_eq!(
        md.mem_size(SizeFlags::default()),
        42u32.mem_size(SizeFlags::default()),
    );
    assert_eq!(
        md.mem_size(SizeFlags::default()),
        core::mem::size_of::<u32>(),
    );
}

#[test]
fn test_maybe_dangling_heap_payload() {
    let inner = vec![1i32, 2, 3];
    let md = MaybeDangling::new(inner.clone());
    // Forwards into the wrapped value, following its heap allocation.
    assert_eq!(
        md.mem_size(SizeFlags::default()),
        inner.mem_size(SizeFlags::default()),
    );
}

#[test]
fn test_maybe_dangling_capacity_flag() {
    let mut inner: Vec<i32> = Vec::with_capacity(16);
    inner.extend([1, 2, 3]);
    // `Vec::clone` does not preserve capacity, so capture the reference size
    // before moving the original (capacity-16) vector into the wrapper.
    let reference_size = inner.mem_size(SizeFlags::CAPACITY);
    let md = MaybeDangling::new(inner);
    // The CAPACITY flag must pass through to the wrapped `Vec`.
    assert_eq!(md.mem_size(SizeFlags::CAPACITY), reference_size);
    assert_ne!(
        md.mem_size(SizeFlags::CAPACITY),
        md.mem_size(SizeFlags::default()),
    );
}

#[derive(MemSize, MemDbg)]
struct MaybeDanglingFields {
    flat: MaybeDangling<u64>,
    heap: MaybeDangling<Vec<u8>>,
}

#[test]
fn test_maybe_dangling_fields_derive_mem_dbg() {
    let fields = MaybeDanglingFields {
        flat: MaybeDangling::new(1),
        heap: MaybeDangling::new(vec![1, 2, 3]),
    };

    let mut output = String::new();
    fields
        .mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");

    assert!(
        output.contains("MaybeDanglingFields"),
        "missing struct name:\n{output}"
    );
    for field in ["flat", "heap"] {
        let needle = format!("╴{field}:");
        assert_eq!(
            output.matches(&needle).count(),
            1,
            "expected one `{needle}` line, got:\n{output}"
        );
    }
}
