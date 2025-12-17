//! Test suite to verify whether derive works properly when MemSize is not imported outside of the derive.

#[allow(dead_code)]
#[cfg_attr(feature = "derive", derive(mem_dbg::MemSize))]
#[cfg_attr(all(feature = "std", feature = "derive"), derive(mem_dbg::MemDbg))]
struct MyTestStruct(i32);

#[test]
#[cfg(feature = "derive")]
fn test_mem_size_no_import() {
    let my_test_struct = MyTestStruct(42);
    let mem_size = <MyTestStruct as mem_dbg::MemSize>::mem_size(
        &my_test_struct,
        mem_dbg::SizeFlags::default(),
    );
    assert_eq!(mem_size, 4);
}
