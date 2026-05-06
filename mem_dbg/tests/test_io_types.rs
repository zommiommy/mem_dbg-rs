#![cfg(feature = "std")]

//! Precise size assertions for `BufReader<T>`, `BufWriter<T>`, `Cursor<T>`,
//! plus the file-system flat types (`File`, `OpenOptions`, `Metadata`,
//! `FileType`, `FileTimes`, `Permissions`).

use mem_dbg::*;
use std::io::{BufReader, BufWriter, Cursor};

#[test]
fn test_cursor_flat_payload() {
    let data: Vec<u8> = (0..16).collect();
    let len = data.len();
    let cur = Cursor::new(data);
    // Cursor<Vec<u8>>: stack frame minus T plus T::mem_size_rec(inner).
    // Vec<u8> inner = size_of::<Vec<u8>>() + len.
    assert_eq!(
        cur.mem_size(SizeFlags::default()),
        core::mem::size_of::<Cursor<Vec<u8>>>() - core::mem::size_of::<Vec<u8>>()
            + core::mem::size_of::<Vec<u8>>()
            + len
    );
}

#[test]
fn test_cursor_string_payload() {
    let s = String::from("hello, world");
    let len = s.len();
    let cur = Cursor::new(s);
    assert_eq!(
        cur.mem_size(SizeFlags::default()),
        core::mem::size_of::<Cursor<String>>() - core::mem::size_of::<String>()
            + core::mem::size_of::<String>()
            + len
    );
}

#[test]
fn test_bufreader_capacity() {
    use std::io::Cursor as IoCursor;

    let data: Vec<u8> = (0..32).collect();
    let inner = IoCursor::new(data);
    let buffered = BufReader::new(inner);

    // BufReader<T>::mem_size = size_of::<Self>() - size_of::<T>() + capacity()
    //                          + T::mem_size_rec(inner).
    let inner_self_size = core::mem::size_of::<IoCursor<Vec<u8>>>();
    // Cursor<Vec<u8>>::mem_size_rec under default flags: stack of Cursor +
    // Vec<u8>'s heap (len bytes when no CAPACITY flag).
    let inner_size = inner_self_size + 32;
    assert_eq!(
        buffered.mem_size(SizeFlags::default()),
        core::mem::size_of::<BufReader<IoCursor<Vec<u8>>>>() - inner_self_size
            + buffered.capacity()
            + inner_size
    );
}

#[test]
fn test_bufwriter_capacity() {
    let inner: Vec<u8> = Vec::new();
    let inner_size = core::mem::size_of::<Vec<u8>>();
    let buffered = BufWriter::new(inner);

    assert_eq!(
        buffered.mem_size(SizeFlags::default()),
        core::mem::size_of::<BufWriter<Vec<u8>>>() - inner_size + buffered.capacity() + inner_size
    );
}

#[test]
#[cfg_attr(miri, ignore)] // miri can't open real files
fn test_fs_flat_types() {
    use std::fs;

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let f = fs::File::open(&path).expect("open Cargo.toml");
    let opts = fs::OpenOptions::new();
    let meta = f.metadata().expect("metadata");
    let ft = meta.file_type();
    let perm = meta.permissions();

    assert_eq!(
        f.mem_size(SizeFlags::default()),
        core::mem::size_of::<fs::File>()
    );
    assert_eq!(
        opts.mem_size(SizeFlags::default()),
        core::mem::size_of::<fs::OpenOptions>()
    );
    assert_eq!(
        meta.mem_size(SizeFlags::default()),
        core::mem::size_of::<fs::Metadata>()
    );
    assert_eq!(
        ft.mem_size(SizeFlags::default()),
        core::mem::size_of::<fs::FileType>()
    );
    assert_eq!(
        perm.mem_size(SizeFlags::default()),
        core::mem::size_of::<fs::Permissions>()
    );
}

#[test]
#[cfg_attr(miri, ignore)] // FileTimes() requires real time system calls on some platforms
fn test_filetimes() {
    let ft = std::fs::FileTimes::new();
    assert_eq!(
        ft.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::fs::FileTimes>()
    );
}
