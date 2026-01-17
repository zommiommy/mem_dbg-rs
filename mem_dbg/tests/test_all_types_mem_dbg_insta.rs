#![cfg(all(feature = "std", feature = "derive"))]

use insta::assert_snapshot;
use mem_dbg::*;

mod all_types_helper;
use all_types_helper::run_all_types_test;

#[test]
fn test_all_types_mem_dbg_snapshot() {
    let combinations = vec![
        ("default", DbgFlags::default()),
        ("empty", DbgFlags::empty()),
        ("capacity", DbgFlags::default() | DbgFlags::CAPACITY),
        ("humanize", DbgFlags::default() | DbgFlags::HUMANIZE),
        ("follow_refs", DbgFlags::default() | DbgFlags::FOLLOW_REFS),
    ];

    for (name, flags) in combinations {
        let output = run_all_types_test(|all_types| {
            let mut output = String::new();
            all_types
                .mem_dbg_on(&mut output, flags)
                .expect("mem_dbg_on failed");
            output
        });
        assert_snapshot!(name, output);
    }
}
