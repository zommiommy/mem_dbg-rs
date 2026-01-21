#![cfg(feature = "mmap-rs")]
#![cfg(feature = "derive")]
use mem_dbg::{SizeFlags, *};

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

    for flag in [
        SizeFlags::default(),
        SizeFlags::FOLLOW_RCS,
        SizeFlags::FOLLOW_REFS,
        SizeFlags::CAPACITY,
    ] {
        let size = s.mem_size(flag);
        assert!(size > 0);
    }
    for flag in [
        DbgFlags::default(),
        DbgFlags::FOLLOW_RCS,
        DbgFlags::FOLLOW_REFS,
        DbgFlags::CAPACITY,
    ] {
        assert!(s.mem_dbg(flag).is_ok());

        for depth in 0..3 {
            let result = s.mem_dbg_depth(depth, flag);
            assert!(result.is_ok());
        }
    }
}
