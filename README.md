# mem_dbg
A trait + derive to recursively compute / print the memory usage of a data structure.  

This is **not** a new idea, but many crates and re-implement them.
This crate is an attempt to unify them with an easy derive.

Examples:
- [bsuccint-rs](https://github.com/beling/bsuccinct-rs/blob/0d7adc7062c0fac6673f8af9def50a799349cfd8/dyn_size_of/src/lib.rs#L7)
