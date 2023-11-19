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
