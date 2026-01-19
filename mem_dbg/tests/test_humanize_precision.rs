#![cfg(all(feature = "std", feature = "derive"))]
use mem_dbg::*;

#[derive(MemSize, MemDbg, Clone, Copy)]
struct Wrapper<const N: usize>([u8; N]);

#[test]
fn test_humanize_precision() {
    let mut s = String::new();

    // Case 1: 1000 bytes -> 1.000 kB (value 1.0, precision 3)
    let w1000 = Wrapper([0u8; 1000]);
    w1000.mem_dbg_on(&mut s, DbgFlags::HUMANIZE).unwrap();
    assert!(s.contains("1.000 kB"));
    s.clear();

    // Case 2: 1234 bytes -> 1.234 kB (value 1.234, precision 3)
    let w1234 = Wrapper([0u8; 1234]);
    w1234.mem_dbg_on(&mut s, DbgFlags::HUMANIZE).unwrap();
    assert!(s.contains("1.234 kB"));
    s.clear();

    // Case 3: 10000 bytes -> 10.00 kB (value 10.0, precision 2)
    let w10000 = Wrapper([0u8; 10000]);
    w10000.mem_dbg_on(&mut s, DbgFlags::HUMANIZE).unwrap();
    assert!(s.contains("10.00 kB"));
    s.clear();

    // Case 4: 999 bytes -> 999 B (value 999, uom B)
    let w999 = Wrapper([0u8; 999]);
    w999.mem_dbg_on(&mut s, DbgFlags::HUMANIZE).unwrap();
    assert!(s.contains("  999  B")); // padded to 5 chars
    s.clear();
}
