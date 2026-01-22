//! Insta snapshot tests for reference deduplication in MemDbg.
//!
//! This test demonstrates how shared references are tracked and displayed:
//! - First encounter: full info with `@0x[ADDR]` marker
//! - Subsequent encounters: abbreviated form with `→ @0x[ADDR]`

#![cfg(all(feature = "std", feature = "derive"))]

use insta::assert_snapshot;
use mem_dbg::*;
use regex::Regex;
use std::rc::Rc;
use std::sync::Arc;

/// Redact dynamic pointer addresses to make snapshots deterministic.
/// Replaces each unique address with a numbered placeholder (e.g., @ 0x[ADDR_1], → 0x[ADDR_1])
/// so that we can verify that shared references have the same address and different ones don't.
fn redact_addresses(s: &str) -> String {
    use std::collections::HashMap;
    // Capture the prefix and the hex address separately
    let re = Regex::new(r"(@ |→ )(0x[a-fA-F0-9]{16})").unwrap();
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

#[derive(MemSize, MemDbg)]
struct SharedRef<'a> {
    name: &'static str,
    first: &'a i32,
    second: &'a i32,
    different: &'a i32,
}

#[derive(MemSize, MemDbg)]
struct SharedRc {
    name: &'static str,
    first: Rc<Vec<i32>>,
    second: Rc<Vec<i32>>,
    different: Rc<Vec<i32>>,
}

#[derive(MemSize, MemDbg)]
struct SharedArc {
    name: &'static str,
    first: Arc<String>,
    second: Arc<String>,
    different: Arc<String>,
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_shared_references_dedup() {
    let shared_value = 42;
    let other_value = 100;

    let test = SharedRef {
        name: "test",
        first: &shared_value,
        second: &shared_value, // Same reference as first
        different: &other_value,
    };

    // Without FOLLOW_REFS - just shows pointer sizes, no markers
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("ref_without_follow_refs", output);

    // With FOLLOW_REFS - shows @0x[ADDR] on first encounter, → @0x[ADDR] on subsequent
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_REFS)
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("ref_with_follow_refs", output);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_shared_rc_dedup() {
    let shared = Rc::new(vec![1, 2, 3, 4, 5]);
    let other = Rc::new(vec![10, 20]);

    let test = SharedRc {
        name: "rc_test",
        first: Rc::clone(&shared),
        second: Rc::clone(&shared), // Same Rc as first
        different: other,
    };

    // Without FOLLOW_RCS - just shows Rc sizes
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("rc_without_follow_rcs", output);

    // With FOLLOW_RCS - shows deduplication of shared Rc
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("rc_with_follow_rcs", output);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_shared_arc_dedup() {
    let shared = Arc::new(String::from("shared_string"));
    let other = Arc::new(String::from("different"));

    let test = SharedArc {
        name: "arc_test",
        first: Arc::clone(&shared),
        second: Arc::clone(&shared), // Same Arc as first
        different: other,
    };

    // Without FOLLOW_RCS - just shows Arc sizes
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("arc_without_follow_rcs", output);

    // With FOLLOW_RCS - shows deduplication of shared Arc
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("arc_with_follow_rcs", output);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_combined_refs_and_rcs() {
    let value = 123;
    let rc_data = Rc::new(vec![1, 2, 3]);

    #[derive(MemSize, MemDbg)]
    struct Combined<'a> {
        ref1: &'a i32,
        ref2: &'a i32,
        rc1: Rc<Vec<i32>>,
        rc2: Rc<Vec<i32>>,
    }

    let test = Combined {
        ref1: &value,
        ref2: &value,
        rc1: Rc::clone(&rc_data),
        rc2: Rc::clone(&rc_data),
    };

    // With both FOLLOW_REFS and FOLLOW_RCS
    let mut output = String::new();
    test.mem_dbg_on(
        &mut output,
        DbgFlags::default() | DbgFlags::FOLLOW_REFS | DbgFlags::FOLLOW_RCS,
    )
    .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("combined_refs_and_rcs", output);
}

/// Inner struct with fields to test recursion into referenced data
#[derive(MemSize, MemDbg)]
struct Inner {
    x: i32,
    y: i64,
    name: String,
}

#[derive(MemSize, MemDbg)]
struct RefToStruct<'a> {
    first: &'a Inner,
    second: &'a Inner,
    different: &'a Inner,
}

#[derive(MemSize, MemDbg)]
struct RcToStruct {
    first: Rc<Inner>,
    second: Rc<Inner>,
    different: Rc<Inner>,
}

#[derive(MemSize, MemDbg)]
struct ArcToStruct {
    first: Arc<Inner>,
    second: Arc<Inner>,
    different: Arc<Inner>,
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_ref_to_struct_with_fields() {
    let shared = Inner {
        x: 42,
        y: 100,
        name: "shared".to_string(),
    };
    let other = Inner {
        x: 1,
        y: 2,
        name: "other".to_string(),
    };

    let test = RefToStruct {
        first: &shared,
        second: &shared,
        different: &other,
    };

    // Without FOLLOW_REFS
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("ref_struct_without_follow", output);

    // With FOLLOW_REFS - should show Inner's fields
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_REFS)
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("ref_struct_with_follow", output);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_rc_to_struct_with_fields() {
    let shared = Rc::new(Inner {
        x: 42,
        y: 100,
        name: "shared".to_string(),
    });
    let other = Rc::new(Inner {
        x: 1,
        y: 2,
        name: "other".to_string(),
    });

    let test = RcToStruct {
        first: Rc::clone(&shared),
        second: Rc::clone(&shared),
        different: other,
    };

    // Without FOLLOW_RCS
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("rc_struct_without_follow", output);

    // With FOLLOW_RCS - should show Inner's fields
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("rc_struct_with_follow", output);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_arc_to_struct_with_fields() {
    let shared = Arc::new(Inner {
        x: 42,
        y: 100,
        name: "shared".to_string(),
    });
    let other = Arc::new(Inner {
        x: 1,
        y: 2,
        name: "other".to_string(),
    });

    let test = ArcToStruct {
        first: Arc::clone(&shared),
        second: Arc::clone(&shared),
        different: other,
    };

    // Without FOLLOW_RCS
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default())
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("arc_struct_without_follow", output);

    // With FOLLOW_RCS - should show Inner's fields
    let mut output = String::new();
    test.mem_dbg_on(&mut output, DbgFlags::default() | DbgFlags::FOLLOW_RCS)
        .expect("mem_dbg_on failed");
    let output = redact_addresses(&output);
    assert_snapshot!("arc_struct_with_follow", output);
}
