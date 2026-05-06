#![cfg(feature = "mmap-rs")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
#[cfg_attr(miri, ignore)] // mmap-rs uses OS-specific APIs unsupported by miri
fn test_mmap_types() {
    use mmap_rs::{Mmap, MmapMut, MmapOptions};
    use std::fs::OpenOptions;

    const MMAP_LEN: usize = 1024;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/test_mmap_comprehensive")
        .unwrap();
    file.set_len(MMAP_LEN as u64).unwrap();

    let mmap: Mmap = unsafe {
        MmapOptions::new(MMAP_LEN)
            .unwrap()
            .with_file(&file, 0)
            .map()
            .unwrap()
    };
    let mmap_mut: MmapMut = unsafe {
        MmapOptions::new(MMAP_LEN)
            .unwrap()
            .with_file(&file, 0)
            .map_mut()
            .unwrap()
    };

    // Default flags: only the handle is reported.
    assert_eq!(
        mmap.mem_size(SizeFlags::default()),
        core::mem::size_of::<Mmap>()
    );
    assert_eq!(
        mmap_mut.mem_size(SizeFlags::default()),
        core::mem::size_of::<MmapMut>()
    );

    // FOLLOW_REFS adds the mapped pages.
    assert_eq!(
        mmap.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<Mmap>() + MMAP_LEN
    );
    assert_eq!(
        mmap_mut.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<MmapMut>() + MMAP_LEN
    );

    // mem_dbg should not panic for any combination of flags.
    for flag in [
        DbgFlags::default(),
        DbgFlags::FOLLOW_RCS,
        DbgFlags::FOLLOW_REFS,
        DbgFlags::CAPACITY,
    ] {
        assert!(mmap.mem_dbg(flag).is_ok());
        assert!(mmap_mut.mem_dbg(flag).is_ok());
    }
}
