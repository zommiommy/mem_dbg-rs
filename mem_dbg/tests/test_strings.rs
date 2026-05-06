#![cfg(feature = "std")]

//! Precise size assertions for string-like types: `String`, `str`, `OsString`,
//! `OsStr`, `Path`, and `PathBuf`. The `mem_size` for these types reports
//! `size_of::<Self>() + len` by default and `size_of::<Self>() + capacity`
//! under `SizeFlags::CAPACITY`. DST variants (`str`, `OsStr`, `Path`) report
//! only their byte length.

use mem_dbg::*;

#[test]
fn test_string() {
    let s = String::from("hello");
    let len = s.len();
    let cap = s.capacity();

    assert_eq!(
        s.mem_size(SizeFlags::default()),
        core::mem::size_of::<String>() + len
    );
    assert_eq!(
        s.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<String>() + cap
    );
}

#[test]
fn test_string_with_extra_capacity() {
    let mut s = String::with_capacity(64);
    s.push_str("hello");
    let len = s.len();
    let cap = s.capacity();
    // The pre-reserved capacity is at least 64; len is just 5.
    assert!(cap >= 64);
    assert_eq!(
        s.mem_size(SizeFlags::default()),
        core::mem::size_of::<String>() + len
    );
    assert_eq!(
        s.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<String>() + cap
    );
}

#[test]
fn test_str_dst() {
    let s: &str = "hello";
    // <str as MemSize>::mem_size returns just the byte length.
    assert_eq!(<str as MemSize>::mem_size(s, SizeFlags::default()), 5);
    let s2: &str = "";
    assert_eq!(<str as MemSize>::mem_size(s2, SizeFlags::default()), 0);
}

#[test]
fn test_str_reference_explicit() {
    let s: &str = "hello";
    // Method-call dispatch on `&str` autoderefs to `<str as MemSize>::mem_size`
    // and reports the byte length. We use FQS to also exercise the `&T` impl,
    // which reports the size of the fat pointer regardless of contents.
    assert_eq!(
        <&str as MemSize>::mem_size(&s, SizeFlags::default()),
        core::mem::size_of::<&str>()
    );
}

#[test]
fn test_pathbuf() {
    let pb = std::path::PathBuf::from("/tmp/some/path");
    let len = pb.as_os_str().len();
    let cap = pb.capacity();
    assert_eq!(
        pb.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::path::PathBuf>() + len
    );
    assert_eq!(
        pb.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<std::path::PathBuf>() + cap
    );
}

#[test]
fn test_path_dst() {
    let pb = std::path::PathBuf::from("/etc/hosts");
    let p: &std::path::Path = pb.as_path();
    let expected = p.as_os_str().len();
    assert_eq!(
        <std::path::Path as MemSize>::mem_size(p, SizeFlags::default()),
        expected
    );
}

#[test]
fn test_osstring() {
    let os = std::ffi::OsString::from("hello");
    let len = os.len();
    let cap = os.capacity();
    assert_eq!(
        os.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::ffi::OsString>() + len
    );
    assert_eq!(
        os.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<std::ffi::OsString>() + cap
    );
}

#[test]
fn test_osstr_dst() {
    let os = std::ffi::OsString::from("hello");
    let osr: &std::ffi::OsStr = os.as_os_str();
    let expected = osr.as_encoded_bytes().len();
    assert_eq!(
        <std::ffi::OsStr as MemSize>::mem_size(osr, SizeFlags::default()),
        expected
    );
}
