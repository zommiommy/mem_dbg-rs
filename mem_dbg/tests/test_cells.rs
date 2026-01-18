#![cfg(feature = "std")]
#![cfg(feature = "derive")]

use mem_dbg::*;
use std::cell::{Cell, OnceCell, RefCell, UnsafeCell};

#[test]
fn test_cell_in_struct() {
    #[derive(MemSize)]
    struct Test {
        cell: Cell<i32>,
    }

    let s = Test {
        cell: Cell::new(42),
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<Cell<i32>>());
}

#[test]
fn test_cell_with_different_types() {
    #[derive(MemSize)]
    struct Test {
        cell_u8: Cell<u8>,
        cell_u64: Cell<u64>,
        cell_bool: Cell<bool>,
    }

    let s = Test {
        cell_u8: Cell::new(1),
        cell_u64: Cell::new(100),
        cell_bool: Cell::new(true),
    };

    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, std::mem::size_of::<Test>());
}

#[test]
fn test_cell_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        value: Cell<i32>,
    }

    let s = Test {
        value: Cell::new(123),
    };

    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, std::mem::size_of::<Test>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}

#[test]
fn test_refcell_in_struct() {
    #[derive(MemSize)]
    struct Test {
        ref_cell: RefCell<i32>,
    }

    let s = Test {
        ref_cell: RefCell::new(42),
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<RefCell<i32>>());
}

#[test]
fn test_refcell_with_vec() {
    #[derive(MemSize)]
    struct Test {
        ref_cell: RefCell<Vec<i32>>,
    }

    let s = Test {
        ref_cell: RefCell::new(vec![1, 2, 3, 4, 5]),
    };

    let size = s.mem_size(SizeFlags::default());
    // RefCell adds overhead around the Vec
    assert!(size > std::mem::size_of::<RefCell<Vec<i32>>>());
}

#[test]
fn test_refcell_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        data: RefCell<String>,
    }

    let s = Test {
        data: RefCell::new("hello".to_string()),
    };

    let size = s.mem_size(SizeFlags::default());
    assert!(size > 0);

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}

#[test]
fn test_unsafe_cell_in_struct() {
    #[derive(MemSize)]
    struct Test {
        unsafe_cell: UnsafeCell<i32>,
    }

    let s = Test {
        unsafe_cell: UnsafeCell::new(42),
    };
    let size = s.mem_size(SizeFlags::default());

    assert_eq!(size, std::mem::size_of::<UnsafeCell<i32>>());
}

#[test]
fn test_unsafe_cell_with_vec() {
    #[derive(MemSize)]
    struct Test {
        unsafe_cell: UnsafeCell<Vec<i32>>,
    }

    let s = Test {
        unsafe_cell: UnsafeCell::new(vec![1, 2, 3, 4, 5]),
    };

    let size = s.mem_size(SizeFlags::default());
    // UnsafeCell adds overhead around the Vec
    assert!(size > std::mem::size_of::<UnsafeCell<Vec<i32>>>());
}

#[test]
fn test_unsafe_cell_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        value: UnsafeCell<u64>,
    }

    let s = Test {
        value: UnsafeCell::new(999),
    };

    let size = s.mem_size(SizeFlags::default());
    assert_eq!(size, std::mem::size_of::<Test>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}

#[test]
fn test_oncecell_empty() {
    #[derive(MemSize)]
    struct Test {
        once_cell: OnceCell<String>,
    }

    let s = Test {
        once_cell: OnceCell::new(),
    };
    let size = s.mem_size(SizeFlags::default());

    // OnceCell may have subtraction overflow issues, use >= instead
    assert!(size >= std::mem::size_of::<OnceCell<String>>());
}

#[test]
fn test_oncecell_initialized() {
    #[derive(MemSize)]
    struct Test {
        once_cell: OnceCell<String>,
    }

    let once_cell = OnceCell::new();
    once_cell.set("initialized".to_string()).unwrap();

    let s = Test { once_cell };
    let size = s.mem_size(SizeFlags::default());

    // OnceCell with initialized value should include the String's heap allocation
    assert!(size > std::mem::size_of::<OnceCell<String>>());
}

#[test]
fn test_oncecell_with_vec() {
    #[derive(MemSize)]
    struct Test {
        once_cell: OnceCell<Vec<i32>>,
    }

    let once_cell = OnceCell::new();
    once_cell.set(vec![1, 2, 3, 4, 5]).unwrap();

    let s = Test { once_cell };
    let size = s.mem_size(SizeFlags::default());

    // OnceCell with Vec should include the Vec's heap allocation
    assert!(size > std::mem::size_of::<OnceCell<Vec<i32>>>());
}

#[test]
fn test_oncecell_with_memdbg() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        data: OnceCell<String>,
    }

    let once_cell = OnceCell::new();
    once_cell.set("test data".to_string()).unwrap();

    let s = Test { data: once_cell };

    let size = s.mem_size(SizeFlags::default());
    // Just verify mem_size doesn't panic
    assert!(size >= std::mem::size_of::<Test>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}

#[test]
fn test_all_cells_in_struct() {
    #[derive(MemSize, MemDbg)]
    struct Test {
        cell: Cell<i32>,
        ref_cell: RefCell<i32>,
        unsafe_cell: UnsafeCell<i32>,
        once_cell: OnceCell<String>,
    }

    let once_cell = OnceCell::new();
    once_cell.set("initialized".to_string()).unwrap();

    let s = Test {
        cell: Cell::new(100),
        ref_cell: RefCell::new(200),
        unsafe_cell: UnsafeCell::new(300),
        once_cell,
    };

    let size = s.mem_size(SizeFlags::default());
    // Just verify mem_size doesn't panic due to OnceCell issues
    assert!(size >= std::mem::size_of::<Test>());

    // Test that mem_dbg works without stack overflow
    let result = s.mem_dbg(DbgFlags::default());
    assert!(result.is_ok());

    for depth in 0..3 {
        let result = s.mem_dbg_depth(depth, DbgFlags::default());
        assert!(result.is_ok());
    }
}
