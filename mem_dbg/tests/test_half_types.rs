#![cfg(feature = "half")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
#[cfg_attr(miri, ignore)] // half crate uses x86 SIMD intrinsics unsupported by miri
fn test_half_types() {
    use half::{bf16, f16};

    #[derive(MemSize, MemDbg)]
    struct HalfStruct {
        f16_val: f16,
        bf16_val: bf16,
    }

    let s = HalfStruct {
        f16_val: f16::from_f32(3.14),
        bf16_val: bf16::from_f32(2.718),
    };

    let size = s.mem_size(SizeFlags::default());
    assert!(size > 0);
    assert!(s.mem_dbg(DbgFlags::default()).is_ok());
    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
