#![cfg(feature = "std")]

//! Precise size assertions for `Result<T, E>`. The `mem_size` follows the
//! same pattern as `Option<T>`: `size_of::<Result<T, E>>()` plus the heap
//! delta of the active variant.

use mem_dbg::*;

#[test]
fn test_result_flat_payloads() {
    let ok: Result<u32, u64> = Ok(7);
    let err: Result<u32, u64> = Err(42);
    let sz = core::mem::size_of::<Result<u32, u64>>();
    // u32 and u64 are flat: heap delta = 0.
    assert_eq!(ok.mem_size(SizeFlags::default()), sz);
    assert_eq!(err.mem_size(SizeFlags::default()), sz);
}

#[test]
fn test_result_ok_with_string() {
    let payload = String::from("hello");
    let len = payload.len();
    let cap = payload.capacity();
    let r: Result<String, u32> = Ok(payload);
    let sz = core::mem::size_of::<Result<String, u32>>();
    assert_eq!(r.mem_size(SizeFlags::default()), sz + len);
    assert_eq!(r.mem_size(SizeFlags::CAPACITY), sz + cap);
}

#[test]
fn test_result_err_with_string() {
    let payload = String::from("oops");
    let len = payload.len();
    let cap = payload.capacity();
    let r: Result<u32, String> = Err(payload);
    let sz = core::mem::size_of::<Result<u32, String>>();
    assert_eq!(r.mem_size(SizeFlags::default()), sz + len);
    assert_eq!(r.mem_size(SizeFlags::CAPACITY), sz + cap);
}

#[test]
fn test_result_both_heap_payloads() {
    let ok = String::from("ok-payload");
    let ok_len = ok.len();
    let r: Result<String, String> = Ok(ok);
    let sz = core::mem::size_of::<Result<String, String>>();
    assert_eq!(r.mem_size(SizeFlags::default()), sz + ok_len);

    let err = String::from("err-payload");
    let err_len = err.len();
    let r: Result<String, String> = Err(err);
    assert_eq!(r.mem_size(SizeFlags::default()), sz + err_len);
}

#[test]
fn test_result_niche_optimised() {
    // `Result<&u8, ()>`: `Err(())` is a ZST so the null pointer encodes it.
    assert_eq!(
        core::mem::size_of::<Result<&u8, ()>>(),
        core::mem::size_of::<&u8>()
    );
    let value = 0u8;
    let ok: Result<&u8, ()> = Ok(&value);
    let err: Result<&u8, ()> = Err(());
    let sz = core::mem::size_of::<Result<&u8, ()>>();
    assert_eq!(ok.mem_size(SizeFlags::default()), sz);
    assert_eq!(err.mem_size(SizeFlags::default()), sz);
}
