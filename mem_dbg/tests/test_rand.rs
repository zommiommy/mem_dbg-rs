#![cfg(feature = "rand")]
#![cfg(feature = "std")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
fn test_rand_types() {
    use rand::SeedableRng;
    use rand::rngs::{SmallRng, StdRng};

    #[derive(MemSize, MemDbg)]
    struct RandStruct {
        small_rng: SmallRng,
        std_rng: StdRng,
    }

    let s = RandStruct {
        small_rng: SmallRng::seed_from_u64(42),
        std_rng: StdRng::seed_from_u64(42),
    };

    let size = s.mem_size(SizeFlags::default());
    assert!(size > 0);
    assert!(s.mem_dbg(DbgFlags::default()).is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
