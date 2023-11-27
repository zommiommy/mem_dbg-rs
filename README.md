# mem_dbg

Traits and associated procedural macros to recursively compute the memory usage of a data structure or print its layout.

The standard library function [`std::mem::size_of`] returns the stack size of a type in bytes, but
it does not take into consideration heap memory. This crate provides a trait [`MemSize`] and an associated procedural macro
to that purpose. 
Moreover, the trait [`MemDbg`] can be used to display the recursive layout of a data structure.

### Example
```rust
use mem_dbg::*;

#[derive(MemSize, MemDbg)]
struct PersonVec<A, B> {
    a: A,
    b: B,
    test: isize,
}

#[derive(MemSize, MemDbg)]
struct Data<A> {
    a: A,
    b: Vec<i32>,
}

#[derive(MemSize, MemDbg)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Unamed(usize, u8),
    Named { first: usize, second: u8 },
}

let person = PersonVec {
    a: TestEnum::Unamed(0, 16),
    b: Data {
        a: vec![0x42_u8; 700],
        b: vec![0xbadf00d; 1000],
    },
    test: -0xbadf00d,
};

// get the type name of the value
// print the size in bytes of the value
println!("mem_size: {}\n", person.mem_size(SizeFlags::default()));

// print the tree of fields and their memory size
person.mem_dbg(DbgFlags::default()).unwrap();
```

it prints:
```text
4_772 B ⏺ : example::PersonVec<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
   16 B ├╴a : example::TestEnum
        │├╴Variant: Unamed
    8 B │├╴0 : usize
    1 B │╰╴1 : u8
4_748 B ├╴b : example::Data<alloc::vec::Vec<u8>>
  724 B │├╴a : alloc::vec::Vec<u8>
4_024 B │╰╴b : alloc::vec::Vec<i32>
    8 B ╰╴test : isize
```
using the flag `DbgFlags::HUMANIZE` it prints:
```text
4.772 KB ⏺ : example::PersonVec<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
   16 B  ├╴a example::TestEnum
         │├╴Variant: Unamed
    8 B  │├╴0 usize
    1 B  │╰╴1 u8
4.748 KB ├╴b example::Data<alloc::vec::Vec<u8>>
  724 B  │├╴a alloc::vec::Vec<u8>
4.024 KB │╰╴b alloc::vec::Vec<i32>
    8 B  ╰╴test isize
```
using the flag `DbgFlags::PERCENTAGE` it prints:
```text
100.00% ⏺ : example::PersonVec<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
  0.34% ├╴a : example::TestEnum
        │├╴Variant: Unamed
  0.17% │├╴0 : usize
  0.02% │╰╴1 : u8
 99.50% ├╴b : example::Data<alloc::vec::Vec<u8>>
 15.17% │├╴a : alloc::vec::Vec<u8>
 84.33% │╰╴b : alloc::vec::Vec<i32>
  0.17% ╰╴test : isize
```

## Caveats
* The data of vectors and slices are not expanded recursively as it might print too much
* `HashMaps`, `HashSet`, `BTreeMap`, `BTreeSet`, are not currently supported as we still have to figure out a way to precisely measure their memory usage