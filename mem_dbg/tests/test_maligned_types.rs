#![cfg(feature = "maligned")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
fn test_maligned_alignment_markers() {
    use maligned::{A2, A4, A8, A16, A32, A64, A128, A256, A512};

    let a2 = A2::default();
    let a4 = A4::default();
    let a8 = A8::default();
    let a16 = A16::default();
    let a32 = A32::default();
    let a64 = A64::default();
    let a128 = A128::default();
    let a256 = A256::default();
    let a512 = A512::default();

    assert_eq!(
        a2.mem_size(SizeFlags::default()),
        core::mem::size_of::<A2>()
    );
    assert_eq!(
        a4.mem_size(SizeFlags::default()),
        core::mem::size_of::<A4>()
    );
    assert_eq!(
        a8.mem_size(SizeFlags::default()),
        core::mem::size_of::<A8>()
    );
    assert_eq!(
        a16.mem_size(SizeFlags::default()),
        core::mem::size_of::<A16>()
    );
    assert_eq!(
        a32.mem_size(SizeFlags::default()),
        core::mem::size_of::<A32>()
    );
    assert_eq!(
        a64.mem_size(SizeFlags::default()),
        core::mem::size_of::<A64>()
    );
    assert_eq!(
        a128.mem_size(SizeFlags::default()),
        core::mem::size_of::<A128>()
    );
    assert_eq!(
        a256.mem_size(SizeFlags::default()),
        core::mem::size_of::<A256>()
    );
    assert_eq!(
        a512.mem_size(SizeFlags::default()),
        core::mem::size_of::<A512>()
    );
}

#[test]
fn test_maligned_aligned_flat_payload() {
    use maligned::{A16, Aligned};
    let a: Aligned<A16, u32> = Aligned::new(42);
    // Flat payload: no heap delta, full size is the aligned wrapper itself.
    assert_eq!(
        a.mem_size(SizeFlags::default()),
        core::mem::size_of::<Aligned<A16, u32>>()
    );
}

#[test]
fn test_maligned_aligned_heap_payload() {
    use maligned::{A64, Aligned};
    let v: Aligned<A64, Vec<i32>> = Aligned::new(vec![1, 2, 3]);
    let len = 3;
    // Aligned<A, T>::mem_size = size_of::<Self>() - size_of::<T>() + T::mem_size_rec(inner).
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        core::mem::size_of::<Aligned<A64, Vec<i32>>>() - core::mem::size_of::<Vec<i32>>()
            + core::mem::size_of::<Vec<i32>>()
            + len * core::mem::size_of::<i32>()
    );
}
