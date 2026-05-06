use mem_dbg::*;

struct TooSmallPadding(u64);

impl FlatType for TooSmallPadding {
    type Flat = True;
}

impl MemSize for TooSmallPadding {
    fn mem_size_rec(&self, _flags: SizeFlags, _refs: &mut HashMap<usize, usize>) -> usize {
        core::mem::size_of_val(&self.0)
    }
}

impl MemDbgImpl for TooSmallPadding {
    fn _mem_dbg_depth_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        max_depth: usize,
        prefix: &mut String,
        field_name: Option<&str>,
        is_last: bool,
        _padded_size: usize,
        flags: DbgFlags,
        dbg_refs: &mut HashSet<usize>,
    ) -> core::fmt::Result {
        self._mem_dbg_depth_on_impl(
            writer,
            total_size,
            max_depth,
            prefix,
            field_name,
            is_last,
            0,
            flags,
            dbg_refs,
            RefDisplay::None,
        )
    }
}

#[test]
fn too_small_padded_size_does_not_underflow() {
    let value = TooSmallPadding(0);
    let mut out = String::new();

    value
        .mem_dbg_on(&mut out, DbgFlags::default())
        .expect("mem_dbg_on should saturate a too-small padded_size");

    assert!(out.contains('⏺'));
    assert!(!out.contains('['), "unexpected padding marker:\n{out}");
}
