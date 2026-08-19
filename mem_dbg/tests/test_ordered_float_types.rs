#![allow(clippy::approx_constant)]
#![cfg(feature = "ordered-float")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;
use ordered_float::{NotNan, OrderedFloat};

#[test]
#[cfg(feature = "half")]
#[cfg_attr(miri, ignore)] // half crate uses x86 SIMD intrinsics unsupported by miri
fn test_ordered_half_types() -> Result<(), anyhow::Error> {
    use half::{bf16, f16};

    let not_nan_f16_val = NotNan::new(f16::from_f32(3.14))?;
    let not_nan_bf16_val = NotNan::new(bf16::from_f32(2.718))?;

    assert_eq!(
        not_nan_f16_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<NotNan<f16>>()
    );
    assert_eq!(not_nan_f16_val.mem_size(SizeFlags::default()), 2);
    assert_eq!(
        not_nan_bf16_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<NotNan<bf16>>()
    );
    assert_eq!(not_nan_bf16_val.mem_size(SizeFlags::default()), 2);

    let ordered_f16_val = OrderedFloat(f16::from_f32(3.14));
    let ordered_bf16_val = OrderedFloat(bf16::from_f32(2.718));

    assert_eq!(
        ordered_f16_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<OrderedFloat<f16>>()
    );
    assert_eq!(ordered_f16_val.mem_size(SizeFlags::default()), 2);
    assert_eq!(
        ordered_bf16_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<OrderedFloat<bf16>>()
    );
    assert_eq!(not_nan_bf16_val.mem_size(SizeFlags::default()), 2);
    Ok(())
}

#[test]
fn test_ordered_float_types() -> Result<(), anyhow::Error> {
    let not_nan_f64_val = NotNan::new(3.14_f64)?;
    let not_nan_f32_val = NotNan::new(2.718_f32)?;

    assert_eq!(
        not_nan_f64_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<NotNan<f64>>()
    );
    assert_eq!(not_nan_f64_val.mem_size(SizeFlags::default()), 8);
    assert_eq!(
        not_nan_f32_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<NotNan<f32>>()
    );
    assert_eq!(not_nan_f32_val.mem_size(SizeFlags::default()), 4);

    let ordered_f64_val = OrderedFloat(3.14_f64);
    let ordered_f32_val = OrderedFloat(2.718_f32);

    assert_eq!(
        ordered_f64_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<OrderedFloat<f64>>()
    );
    assert_eq!(ordered_f64_val.mem_size(SizeFlags::default()), 8);
    assert_eq!(
        ordered_f32_val.mem_size(SizeFlags::default()),
        core::mem::size_of::<OrderedFloat<f32>>()
    );
    assert_eq!(not_nan_f32_val.mem_size(SizeFlags::default()), 4);
    Ok(())
}
