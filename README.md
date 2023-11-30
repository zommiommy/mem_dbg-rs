# mem_dbg

Traits and associated procedural macros to recursively compute the memory usage of a data structure or print its layout.

The standard library function [`std::mem::size_of`] returns the stack size of a type in bytes, but
it does not take into consideration heap memory. This crate provides a trait [`MemSize`] and an associated procedural macro
for that purpose. Moreover, the trait [`MemDbg`] can be used to display the recursive layout of a data structure, together
with the size of each part.

Both traits accept options that can change their behavior (e.g., follow references or not) or 
the way the data is displayed (e.g., humanize the size or display it as a percentage of the total size).

### Example
```rust
use mem_dbg::*;

#[derive(MemSize, MemDbg)]
struct Struct<A, B> {
    a: A,
    b: B,
    test: isize,
}

#[derive(MemSize, MemDbg)]
struct Data<A> {
    a: A,
    b: Vec<i32>,
    c: (usize, String)
}

#[derive(MemSize, MemDbg)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Unnamed(usize, u8),
    Named { first: usize, second: u8 },
}

let b = Vec::with_capacity(100);

let s = Struct {
    a: TestEnum::Unnamed(0, 16),
    b: Data {
        a: vec![0x42_u8; 700],
        b,
        c: (1, "foo".to_owned()),
    },
    test: -0xbadf00d,
};

println!("size:     {}", s.mem_size(SizeFlags::default()));
println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));

s.mem_dbg(DbgFlags::default()).unwrap();
```

The previous program prints:
```text
size:     815
capacity: 1215

 815 B 100.00% ⏺: (usize, example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>)
   8 B   0.98% ├╴0: usize
 807 B  99.02% ╰╴1: example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
  16 B   1.96%   ├╴a: example::TestEnum
                 │ ├╴Variant: Unnamed
   8 B   0.98%   │ ├╴0: usize
   1 B   0.12%   │ ╰╴1: u8
 783 B  96.07%   ├╴b: example::Data<alloc::vec::Vec<u8>>
 724 B  88.83%   │ ├╴a: alloc::vec::Vec<u8>
  24 B   2.94%   │ ├╴b: alloc::vec::Vec<i32>
  35 B   4.29%   │ ╰╴c: (usize, alloc::string::String)
   8 B   0.98%   │   ├╴0: usize
  27 B   3.31%   │   ╰╴1: alloc::string::String
   8 B   0.98%   ╰╴test: isize
```
If we add the flags [`DbgFlags::CAPACITY`] and [`DbgFlags::HUMANIZE`] it prints:
```text
size:     815
capacity: 1215

1.215 kB 100.00% ⏺: (usize, example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>)
    8  B   0.66% ├╴0: usize
1.207 kB  99.34% ╰╴1: example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
   16  B   1.32%   ├╴a: example::TestEnum
                   │ ├╴Variant: Unnamed
    8  B   0.66%   │ ├╴0: usize
    1  B   0.08%   │ ╰╴1: u8
1.183 kB  97.37%   ├╴b: example::Data<alloc::vec::Vec<u8>>
  724  B  59.59%   │ ├╴a: alloc::vec::Vec<u8>
  424  B  34.90%   │ ├╴b: alloc::vec::Vec<i32>
   35  B   2.88%   │ ╰╴c: (usize, alloc::string::String)
    8  B   0.66%   │   ├╴0: usize
   27  B   2.22%   │   ╰╴1: alloc::string::String
    8  B   0.66%   ╰╴test: isize
```
If we use [`DbgFlags::empty()`] it prints:
```text
size:     815
capacity: 1215

4815 B ⏺
   8 B ├╴0
4807 B ╰╴1
  16 B  ├╴a
        │├╴Variant: Unnamed
   8 B  │├╴0
   1 B  │╰╴1
4783 B  ├╴b
 724 B  │├╴a
4024 B  │├╴b
  35 B  │╰╴c
   8 B  │ ├╴0
  27 B  │ ╰╴1
   8 B  ╰╴test
```

## Caveats

* We support out-of-the-box most basic types, and tuples up to size ten. The derive macros 
  `MemSize`/`MemDbg` will generate
  implementations for structs and enums whose fields implement the associated interface: if this is not
  the case (e.g., because of the orphan rule) one can implement the traits manually.

* Computation of the size of arrays, slices and vectors will be performed by iterating over their elements
  unless the type is a copy type that does not contain references and it is declared as such using
  the attribute `#[copy_type]`. See [`CopyType`] for more details.

* The content of vectors and slices is not expanded recursively as the output might be too 
  complex; this might change in the future (e.g., via a flag) should interesting use cases arise.

* `HashMaps`, `HashSet`, `BTreeMap`, and `BTreeSet`, are not currently supported as we still 
  have to figure out a way to precisely measure their memory size and capacity.
