#![cfg(feature = "derive")]
#![cfg(feature = "std")]

use core::mem::ManuallyDrop;
use mem_dbg::*;

#[test]
fn test_manually_drop_size() {
    #[derive(MemSize)]
    struct Test {
        data: ManuallyDrop<usize>,
    }

    let t = Test {
        data: ManuallyDrop::new(10),
    };

    // ManuallyDrop<usize> should be size of usize
    assert_eq!(std::mem::size_of::<Test>(), std::mem::size_of::<usize>());
    // mem_size should be size of usize
    assert_eq!(
        t.mem_size(SizeFlags::default()),
        std::mem::size_of::<usize>()
    );
}

#[test]
fn test_manually_drop_dbg() {
    #[derive(MemDbg, MemSize)]
    struct Test {
        data: ManuallyDrop<usize>,
    }

    let t = Test {
        data: ManuallyDrop::new(10),
    };

    // Just check it runs without panic
    t.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
fn test_manually_drop_vec() {
    #[derive(MemSize)]
    struct Test {
        data: ManuallyDrop<Vec<usize>>,
    }

    let t = Test {
        data: ManuallyDrop::new(vec![1, 2, 3]),
    };

    // Size should include heap capacity
    let expected = std::mem::size_of::<Vec<usize>>() + 3 * std::mem::size_of::<usize>();

    let size = t.mem_size(SizeFlags::default());
    assert_eq!(size, expected);
}
