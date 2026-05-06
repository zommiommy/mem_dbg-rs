#![cfg(feature = "rand")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
fn test_rand_types() {
    use rand::SeedableRng;
    use rand::rngs::{SmallRng, StdRng};

    let small_rng = SmallRng::seed_from_u64(42);
    let std_rng = StdRng::seed_from_u64(42);

    // All RNGs are flat: size matches their stack layout.
    assert_eq!(
        small_rng.mem_size(SizeFlags::default()),
        core::mem::size_of::<SmallRng>()
    );
    assert_eq!(
        std_rng.mem_size(SizeFlags::default()),
        core::mem::size_of::<StdRng>()
    );
}
