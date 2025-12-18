#![cfg(feature = "mmap-rs")]
#![cfg(feature = "derive")]
use mem_dbg::*;

#[test]
fn test_mmap_types() {
    use mmap_rs::{Mmap, MmapMut, MmapOptions};
    use std::fs::OpenOptions;

    #[derive(MemSize, MemDbg)]
    struct MmapStruct {
        mmap: Mmap,
        mmap_mut: MmapMut,
    }

    // Create a temp file for mmap
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("/tmp/test_mmap_comprehensive")
        .unwrap();
    file.set_len(1024).unwrap();

    let s = MmapStruct {
        mmap: unsafe {
            MmapOptions::new(1024)
                .unwrap()
                .with_file(&file, 0)
                .map()
                .unwrap()
        },
        mmap_mut: unsafe {
            MmapOptions::new(1024)
                .unwrap()
                .with_file(&file, 0)
                .map_mut()
                .unwrap()
        },
    };

    let size = s.mem_size(SizeFlags::default());
    assert!(size > 0);
    assert!(s.mem_dbg(DbgFlags::default()).is_ok());
}
