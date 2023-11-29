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

let person = Struct {
    a: TestEnum::Unnamed(0, 16),
    b: Data {
        a: vec![0x42_u8; 700],
        b: vec![0xbadf00d; 1000],
        c: (1, "foo".to_owned()),
    },
    test: -0xbadf00d,
};

// print the size in bytes of the value
println!("size: {}\n", person.mem_size(SizeFlags::default()));

// print the tree of fields and their memory size
person.mem_dbg(DbgFlags::default()).unwrap();
```

The previous program prints:
```text
size: 4815

4_815 B 100.00% ⏺ : (usize, example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>)
    8 B   0.17% ├╴0 : usize
4_807 B  99.83% ╰╴1 : example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
   16 B   0.33%  ├╴a : example::TestEnum
                 │├╴Variant: Unnamed
    8 B   0.17%  │├╴0 : usize
    1 B   0.02%  │╰╴1 : u8
4_783 B  99.34%  ├╴b : example::Data<alloc::vec::Vec<u8>>
  724 B  15.04%  │├╴a : alloc::vec::Vec<u8>
4_024 B  83.57%  │├╴b : alloc::vec::Vec<i32>
   35 B   0.73%  │╰╴c : (usize, alloc::string::String)
    8 B   0.17%  ││├╴0 : usize
   27 B   0.56%  ││╰╴1 : alloc::string::String
    8 B   0.17%  ╰╴test : isize
```
If we use the flag [`DbgFlags::HUMANIZE`] it prints:
```text
size: 4815

4.815 KB100.00% ⏺ : (usize, example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>)
     8 B   0.17% ├╴0 : usize
4.807 KB 99.83% ╰╴1 : example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
    16 B   0.33%  ├╴a : example::TestEnum
                  │├╴Variant: Unnamed
     8 B   0.17%  │├╴0 : usize
     1 B   0.02%  │╰╴1 : u8
4.783 KB 99.34%  ├╴b : example::Data<alloc::vec::Vec<u8>>
   724 B  15.04%  │├╴a : alloc::vec::Vec<u8>
4.024 KB 83.57%  │├╴b : alloc::vec::Vec<i32>
    35 B   0.73%  │╰╴c : (usize, alloc::string::String)
     8 B   0.17%  ││├╴0 : usize
    27 B   0.56%  ││╰╴1 : alloc::string::String
     8 B   0.17%  ╰╴test : isize
```
If we use no flags it prints:
```text
size: 4815

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
   8 B  ││├╴0
  27 B  ││╰╴1
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
