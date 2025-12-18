#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;

#[test]
fn test_box_in_struct() {
    #[derive(MemSize)]
    struct Test {
        boxed: Box<u64>,
    }

    let s = Test {
        boxed: Box::new(42),
    };
    let size = s.mem_size(SizeFlags::default());

    // Box adds its pointer size + the contained value
    let expected = std::mem::size_of::<Box<u64>>() + std::mem::size_of::<u64>();
    assert_eq!(size, expected);
}
