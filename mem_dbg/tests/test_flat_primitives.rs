//! Precise size assertions for every flat (`size_of`-only) `MemSize` impl that
//! is not already covered by a dedicated test file: primitive integers, floats,
//! `bool`, `char`, `()`, hash-state types, raw pointers, `NonNull`, `Weak`,
//! function pointers, `Layout`, network types, and time types. Each assertion
//! checks `value.mem_size(default) == size_of::<value>()` since these types
//! are byte-identical to their stack footprint.

use core::ptr::NonNull;
use mem_dbg::*;

#[test]
fn test_unit() {
    assert_eq!(().mem_size(SizeFlags::default()), 0);
}

#[test]
fn test_bool() {
    assert_eq!(false.mem_size(SizeFlags::default()), 1);
    assert_eq!(true.mem_size(SizeFlags::default()), 1);
}

#[test]
fn test_char() {
    assert_eq!('a'.mem_size(SizeFlags::default()), 4);
    assert_eq!(
        '\u{1F980}'.mem_size(SizeFlags::default()),
        core::mem::size_of::<char>()
    );
}

#[test]
fn test_floats() {
    assert_eq!(0.0_f32.mem_size(SizeFlags::default()), 4);
    assert_eq!(0.0_f64.mem_size(SizeFlags::default()), 8);
}

#[test]
fn test_signed_integers() {
    assert_eq!(0_i8.mem_size(SizeFlags::default()), 1);
    assert_eq!(0_i16.mem_size(SizeFlags::default()), 2);
    assert_eq!(0_i32.mem_size(SizeFlags::default()), 4);
    assert_eq!(0_i64.mem_size(SizeFlags::default()), 8);
    assert_eq!(0_i128.mem_size(SizeFlags::default()), 16);
    assert_eq!(
        0_isize.mem_size(SizeFlags::default()),
        core::mem::size_of::<isize>()
    );
}

#[test]
fn test_unsigned_integers() {
    assert_eq!(0_u8.mem_size(SizeFlags::default()), 1);
    assert_eq!(0_u16.mem_size(SizeFlags::default()), 2);
    assert_eq!(0_u32.mem_size(SizeFlags::default()), 4);
    assert_eq!(0_u64.mem_size(SizeFlags::default()), 8);
    assert_eq!(0_u128.mem_size(SizeFlags::default()), 16);
    assert_eq!(
        0_usize.mem_size(SizeFlags::default()),
        core::mem::size_of::<usize>()
    );
}

#[test]
fn test_raw_pointers() {
    let value = 7u32;
    let const_ptr: *const u32 = &value;
    let mut owned = 7u32;
    let mut_ptr: *mut u32 = &mut owned;
    assert_eq!(
        const_ptr.mem_size(SizeFlags::default()),
        core::mem::size_of::<*const u32>()
    );
    assert_eq!(
        mut_ptr.mem_size(SizeFlags::default()),
        core::mem::size_of::<*mut u32>()
    );
    // FOLLOW_REFS does not apply to raw pointers; size must stay flat.
    assert_eq!(
        const_ptr.mem_size(SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<*const u32>()
    );
}

#[test]
fn test_nonnull() {
    let mut value = 7u32;
    let nn = NonNull::from(&mut value);
    assert_eq!(
        nn.mem_size(SizeFlags::default()),
        core::mem::size_of::<NonNull<u32>>()
    );
}

#[test]
fn test_weak_pointers() {
    let rc_weak: std::rc::Weak<u32> = std::rc::Weak::new();
    let arc_weak: std::sync::Weak<u32> = std::sync::Weak::new();
    assert_eq!(
        rc_weak.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::rc::Weak<u32>>()
    );
    // Weak is never followed even when FOLLOW_RCS is set.
    assert_eq!(
        rc_weak.mem_size(SizeFlags::FOLLOW_RCS),
        core::mem::size_of::<std::rc::Weak<u32>>()
    );
    assert_eq!(
        arc_weak.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::sync::Weak<u32>>()
    );
    assert_eq!(
        arc_weak.mem_size(SizeFlags::FOLLOW_RCS),
        core::mem::size_of::<std::sync::Weak<u32>>()
    );
}

#[test]
fn test_function_pointers() {
    fn f0() -> i32 {
        0
    }
    fn f1(_a: i32) -> i32 {
        0
    }
    fn f2(_a: i32, _b: i32) -> i32 {
        0
    }
    fn f3(_a: i32, _b: i32, _c: i32) -> i32 {
        0
    }
    let p0 = f0 as fn() -> i32;
    let p1 = f1 as fn(i32) -> i32;
    let p2 = f2 as fn(i32, i32) -> i32;
    let p3 = f3 as fn(i32, i32, i32) -> i32;
    assert_eq!(
        p0.mem_size(SizeFlags::default()),
        core::mem::size_of::<fn() -> i32>()
    );
    assert_eq!(
        p1.mem_size(SizeFlags::default()),
        core::mem::size_of::<fn(i32) -> i32>()
    );
    assert_eq!(
        p2.mem_size(SizeFlags::default()),
        core::mem::size_of::<fn(i32, i32) -> i32>()
    );
    assert_eq!(
        p3.mem_size(SizeFlags::default()),
        core::mem::size_of::<fn(i32, i32, i32) -> i32>()
    );
}

#[test]
fn test_layout() {
    let layout = core::alloc::Layout::new::<u64>();
    assert_eq!(
        layout.mem_size(SizeFlags::default()),
        core::mem::size_of::<core::alloc::Layout>()
    );
}

#[test]
fn test_default_hasher() {
    let hasher = std::collections::hash_map::DefaultHasher::new();
    assert_eq!(
        hasher.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::collections::hash_map::DefaultHasher>()
    );
}

#[test]
fn test_random_state() {
    let state = std::collections::hash_map::RandomState::new();
    assert_eq!(
        state.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::collections::hash_map::RandomState>()
    );
}

#[test]
fn test_build_hasher_default() {
    let bhd: core::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher> =
        core::hash::BuildHasherDefault::default();
    assert_eq!(bhd.mem_size(SizeFlags::default()), 0);
    assert_eq!(
        core::mem::size_of::<
            core::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>,
        >(),
        0,
    );
}

#[test]
fn test_net_addresses() {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
    let v4 = Ipv4Addr::new(127, 0, 0, 1);
    let v6 = Ipv6Addr::LOCALHOST;
    let ip4 = IpAddr::V4(v4);
    let ip6 = IpAddr::V6(v6);
    let sav4 = SocketAddrV4::new(v4, 8080);
    let sav6 = SocketAddrV6::new(v6, 8080, 0, 0);
    let sa = SocketAddr::V4(sav4);

    assert_eq!(
        v4.mem_size(SizeFlags::default()),
        core::mem::size_of::<Ipv4Addr>()
    );
    assert_eq!(
        v6.mem_size(SizeFlags::default()),
        core::mem::size_of::<Ipv6Addr>()
    );
    assert_eq!(
        ip4.mem_size(SizeFlags::default()),
        core::mem::size_of::<IpAddr>()
    );
    assert_eq!(
        ip6.mem_size(SizeFlags::default()),
        core::mem::size_of::<IpAddr>()
    );
    assert_eq!(
        sav4.mem_size(SizeFlags::default()),
        core::mem::size_of::<SocketAddrV4>()
    );
    assert_eq!(
        sav6.mem_size(SizeFlags::default()),
        core::mem::size_of::<SocketAddrV6>()
    );
    assert_eq!(
        sa.mem_size(SizeFlags::default()),
        core::mem::size_of::<SocketAddr>()
    );
}

#[cfg_attr(miri, ignore)] // miri lacks REALTIME clock_gettime
#[test]
fn test_time_types() {
    use std::time::{Duration, Instant, SystemTime};
    let d = Duration::from_secs(1);
    let i = Instant::now();
    let s = SystemTime::now();

    assert_eq!(
        d.mem_size(SizeFlags::default()),
        core::mem::size_of::<Duration>()
    );
    assert_eq!(
        i.mem_size(SizeFlags::default()),
        core::mem::size_of::<Instant>()
    );
    assert_eq!(
        s.mem_size(SizeFlags::default()),
        core::mem::size_of::<SystemTime>()
    );
}

#[cfg_attr(miri, ignore)] // miri lacks REALTIME clock_gettime
#[test]
fn test_system_time_error() {
    use std::time::{Duration, SystemTime};
    // Construct a SystemTimeError by asking how much the past is "since" the
    // future; this returns Err(SystemTimeError(_)).
    let now = SystemTime::now();
    let future = now + Duration::from_secs(60);
    let err = now
        .duration_since(future)
        .expect_err("future is later than now");
    assert_eq!(
        err.mem_size(SizeFlags::default()),
        core::mem::size_of::<std::time::SystemTimeError>()
    );
}
