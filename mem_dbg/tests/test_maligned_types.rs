#![cfg(feature = "maligned")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
fn test_maligned_types() {
    use maligned::{A2, A4, A8, A16, A32, A64, A128, A256, A512, Aligned};

    #[derive(MemSize, MemDbg)]
    struct MalignedStruct {
        a2: A2,
        a4: A4,
        a8: A8,
        a16: A16,
        a32: A32,
        a64: A64,
        a128: A128,
        a256: A256,
        a512: A512,
        aligned_u32: Aligned<A16, u32>,
        aligned_vec: Aligned<A64, Vec<i32>>,
    }

    let s = MalignedStruct {
        a2: A2::default(),
        a4: A4::default(),
        a8: A8::default(),
        a16: A16::default(),
        a32: A32::default(),
        a64: A64::default(),
        a128: A128::default(),
        a256: A256::default(),
        a512: A512::default(),
        aligned_u32: Aligned::new(42),
        aligned_vec: Aligned::new(vec![1, 2, 3]),
    };

    let size = s.mem_size(SizeFlags::default());
    assert!(size > 0);
    assert!(s.mem_dbg(DbgFlags::default()).is_ok());
    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
