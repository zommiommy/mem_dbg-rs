#![cfg(all(feature = "std", feature = "derive"))]

use insta::{assert_snapshot, with_settings};
use mem_dbg::*;

#[derive(MemSize, MemDbg)]
struct Inner {
    x: u64,
    y: Vec<u8>,
}

#[derive(MemSize, MemDbg)]
struct Outer {
    a: Inner,
    b: String,
}

fn make_test_value() -> Outer {
    Outer {
        a: Inner {
            x: 42,
            y: vec![1, 2, 3],
        },
        b: "hello".to_owned(),
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_mem_dbg_on_and_depth_on_consistency() {
    let val = make_test_value();
    let flags = DbgFlags::default();
    let arch = std::env::consts::ARCH;

    // mem_dbg_on (unlimited depth)
    let mut on_output = String::new();
    val.mem_dbg_on(&mut on_output, flags).unwrap();

    // mem_dbg_depth_on with usize::MAX (effectively unlimited)
    let mut depth_on_output = String::new();
    val.mem_dbg_depth_on(&mut depth_on_output, usize::MAX, flags)
        .unwrap();

    // Both entry points must produce identical output
    assert_eq!(on_output, depth_on_output);

    with_settings!({snapshot_suffix => arch}, {
        assert_snapshot!("entry_points_full", on_output);
    });
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_mem_dbg_depth_on_depth_0() {
    let val = make_test_value();
    let flags = DbgFlags::default();
    let arch = std::env::consts::ARCH;

    let mut output = String::new();
    val.mem_dbg_depth_on(&mut output, 0, flags).unwrap();

    with_settings!({snapshot_suffix => arch}, {
        assert_snapshot!("entry_points_depth_0", output);
    });
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_mem_dbg_depth_on_depth_1() {
    let val = make_test_value();
    let flags = DbgFlags::default();
    let arch = std::env::consts::ARCH;

    let mut output = String::new();
    val.mem_dbg_depth_on(&mut output, 1, flags).unwrap();

    with_settings!({snapshot_suffix => arch}, {
        assert_snapshot!("entry_points_depth_1", output);
    });
}
