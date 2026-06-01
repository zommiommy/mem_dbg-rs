#![cfg(all(
    feature = "aliasable",
    feature = "maybe-dangling",
    feature = "std",
    feature = "derive"
))]

//! Insta snapshot coverage for the optional-feature wrapper types.

use aliasable::AliasableMut;
use aliasable::boxed::AliasableBox;
use aliasable::string::AliasableString;
use aliasable::vec::AliasableVec;
use insta::{assert_snapshot, with_settings};
use maybe_dangling::MaybeDangling;
use mem_dbg::*;
use regex::Regex;

/// Redact dynamic pointer addresses to make snapshots deterministic, mirroring
/// the helper in `test_all_types_mem_dbg_insta.rs`. Each distinct address maps
/// to a stable numbered placeholder so shared addresses stay visibly shared.
fn redact_addresses(s: &str) -> String {
    use std::collections::HashMap;
    let re = Regex::new(r"(@ |→ )(0x[a-fA-F0-9]{8,16})").unwrap();
    let mut addr_map: HashMap<String, usize> = HashMap::new();
    let mut counter = 1;

    re.replace_all(s, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let hex_addr = caps[2].to_string();
        let id = *addr_map.entry(hex_addr).or_insert_with(|| {
            let id = counter;
            counter += 1;
            id
        });
        format!("{}0x[ADDR_{}]", prefix, id)
    })
    .to_string()
}

/// The `aliasable` wrappers are memory-layout twins of `Box`/`Vec`/`String`/
/// `&mut T`, and `MaybeDangling<T>` is `repr(transparent)` over `T`. This
/// struct exercises one of each, with both a flat and a heap-bearing payload so
/// the snapshots show forwarding into the pointee.
#[derive(MemSize, MemDbg)]
struct OptionalTypes<'a> {
    // aliasable crate
    a_box: AliasableBox<u64>,
    a_vec: AliasableVec<i32>,
    a_vec_str: AliasableVec<String>,
    a_string: AliasableString,
    a_mut: AliasableMut<'a, u64>,

    // maybe-dangling crate
    md_flat: MaybeDangling<u64>,
    md_heap: MaybeDangling<Vec<u8>>,
}

/// Builds an `OptionalTypes` and runs `test` on it. The mutable borrow backing
/// `AliasableMut` is created here so its lifetime outlives the closure.
fn run_optional_types_test<F, R>(test: F) -> R
where
    F: for<'a> FnOnce(OptionalTypes<'a>) -> R,
{
    let mut target = 9u64;
    let value = OptionalTypes {
        a_box: AliasableBox::from_unique(Box::new(12345)),
        a_vec: AliasableVec::from_unique(vec![1, 2, 3, 4, 5]),
        a_vec_str: AliasableVec::from_unique(vec!["alpha".to_string(), "beta".to_string()]),
        a_string: AliasableString::from_unique(String::from("hello")),
        a_mut: AliasableMut::from_unique(&mut target),

        md_flat: MaybeDangling::new(64),
        md_heap: MaybeDangling::new(vec![1u8, 2, 3]),
    };

    test(value)
}

#[test]
#[cfg_attr(miri, ignore)] // consistency with the other snapshot tests
fn test_optional_types_mem_dbg_snapshot() {
    let combinations = vec![
        ("default", DbgFlags::default()),
        ("empty", DbgFlags::empty()),
        ("capacity", DbgFlags::default() | DbgFlags::CAPACITY),
        ("humanize", DbgFlags::default() | DbgFlags::HUMANIZE),
        ("follow_refs", DbgFlags::default() | DbgFlags::FOLLOW_REFS),
    ];

    // Architecture-specific snapshots: pointer-sized wrappers differ between
    // 32- and 64-bit targets, exactly like the AllTypesStruct snapshots.
    let arch = std::env::consts::ARCH;

    for (name, flags) in combinations {
        let output = run_optional_types_test(|value| {
            let mut output = String::new();
            value
                .mem_dbg_on(&mut output, flags)
                .expect("mem_dbg_on failed");
            output
        });
        let output = redact_addresses(&output);
        with_settings!({snapshot_suffix => arch}, {
            assert_snapshot!(name, output);
        });
    }
}
