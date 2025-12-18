#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::path::PathBuf;

#[test]
fn test_pathbuf_in_struct() {
    #[derive(MemSize)]
    struct Test {
        path_buf: PathBuf,
    }

    let s = Test {
        path_buf: PathBuf::from("/tmp/test"),
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, 24);
}
