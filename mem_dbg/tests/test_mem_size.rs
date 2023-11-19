use mem_dbg::MemSize;

#[test]
fn test_vec_capacity() {
    let mut v = vec![Vec::with_capacity(10)];
    v[0].push(1);
    v[0].push(2);
    // We consider the capacity of the inner vector
    assert_eq!(
        8 * std::mem::size_of::<i32>(),
        v.mem_capacity() - v.mem_size()
    );
}

#[test]
fn test_empty_struct() {
    #[derive(MemSize)]
    struct Data {};
    let v = Data {};
    assert_eq!(0, v.mem_size());
    assert_eq!(0, v.mem_capacity());
}

#[test]
fn test_struct() {
    #[derive(MemSize)]
    struct Data {
        a: u64,
        b: Vec<usize>,
    };
    let mut v = Data {
        a: 10,
        b: Vec::with_capacity(10),
    };
    v.b.push(1);
    v.b.push(2);
    assert_eq!(8 + v.b.mem_size(), v.mem_size());
    assert_eq!(8 + v.b.mem_capacity(), v.mem_capacity());
}

#[test]
fn test_empty_tuple_struct() {
    #[derive(MemSize)]
    struct Data();
    let v = Data();
    assert_eq!(0, v.mem_size());
    assert_eq!(0, v.mem_capacity());
}

#[test]
fn test_tuple_struct() {
    #[derive(MemSize)]
    struct Data(u64, Vec<usize>);
    let mut v = Data(10, Vec::with_capacity(10));
    v.1.push(1);
    v.1.push(2);
    assert_eq!(8 + v.1.mem_size(), v.mem_size());
    assert_eq!(8 + v.1.mem_capacity(), v.mem_capacity());
}
