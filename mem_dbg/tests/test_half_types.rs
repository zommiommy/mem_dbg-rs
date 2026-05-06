#![allow(clippy::approx_constant)]
#![cfg(feature = "half")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
#[cfg_attr(miri, ignore)] // half crate uses x86 SIMD intrinsics unsupported by miri
fn test_half_types() {
    use half::{bf16, f16};

    let f16_val = f16::from_f32(3.14);
    let bf16_val = bf16::from_f32(2.718);

    assert_eq!(
        f16_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<f16>()
    );
    assert_eq!(f16_val.mem_size(SizeFlags::default()), 2);
    assert_eq!(
        bf16_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<bf16>()
    );
    assert_eq!(bf16_val.mem_size(SizeFlags::default()), 2);
}
