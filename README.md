# mem_dbg

[![downloads](https://img.shields.io/crates/d/mem_dbg)](https://crates.io/crates/mem_dbg)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/mem_dbg)](https://crates.io/crates/mem_dbg/reverse_dependencies)
![GitHub CI](https://github.com/zommiommy/mem_dbg-rs/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/mem_dbg)
[![](https://tokei.rs/b1/github/zommiommy/mem_dbg-rs?type=Rust,Python)](https://github.com/zommiommy/mem_dbg-rs)
[![Latest version](https://img.shields.io/crates/v/mem_dbg.svg)](https://crates.io/crates/mem_dbg)
[![Documentation](https://docs.rs/mem_dbg/badge.svg)](https://docs.rs/mem_dbg)

Traits and associated procedural macros to display recursively the layout and memory usage of a value.

The trait [`MemDbg`] can be used to display the recursive layout of a value, together with the size of each part. We provide
implementations for most basic types and a derive macro for structs and enums whose fields implement [`MemDbg`].

To compute the size, we provide the trait [`MemSize`] and a derive macro that can be used to compute the size of a value in bytes
as the standard library function [`std::mem::size_of`] returns the stack size of a type in bytes, but
it does not take into consideration heap memory.

# Why `MemSize`

Other traits partially provide the functionality of [`MemSize`], but either they require
implementing manually a trait, which is prone to error, or they do not provide the flexibility necessary
for [`MemDbg`]. Most importantly, [`MemSize`] uses the type system
to avoid iterating over the content of a container (a vector, etc.) when it is not necessary, making it possible to
compute instantly the size of values occupying hundreds of gigabytes of heap memory.

This is the result of the benchmark `bench_hash_map` contained in the `examples` directory. It builds a hash map
with a hundred million entries and then measure its heap size:

```test
Allocated:    2281701509
get_size:     1879048240 152477833 ns
deep_size_of: 1879048240 152482000 ns
size_of:      2281701432 152261958 ns
mem_size:     2281701424 209 ns
```

The first line is the number of bytes allocated by the program as returned by [`cap`].
Then, we display the result of [`get-size`], [`deepsize`],
[`size-of`], and our own [`MemSize`]. Note that the first two crates are just measuring the
space used by the items, and not by the data structure (i.e., they are not taking into account the load factor and the power-of-two size
constraint of the hash map). Moreover, all other crates are about six orders of magnitude slower than our implementation, due to
the necessity to iterate over all elements.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
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

s.mem_dbg(DbgFlags::default())?;
# Ok(())
# }
```

The previous program prints:

```text
size:     815
capacity: 1215

 985 B 100.00% ⏺: example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
  16 B   1.62% ├╴a: example::TestEnum
               │ ├╴Variant: Unnamed
   8 B   0.81% │ ├╴0: usize
   1 B   0.10% │ ╰╴1: u8
 823 B  83.55% ├╴b: example::Data<alloc::vec::Vec<u8>>
 724 B  73.50% │ ├╴a: alloc::vec::Vec<u8>
  64 B   6.50% │ ├╴b: alloc::vec::Vec<i32>
  35 B   3.55% │ ╰╴c: (usize, alloc::string::String)
   8 B   0.81% │   ├╴0: usize
  27 B   2.74% │   ╰╴1: alloc::string::String
   8 B   0.81% ├╴test: isize
 138 B  14.01% ╰╴s: std::collections::hash::set::HashSet<usize>
```

If we add the flags [`DbgFlags::CAPACITY`] and [`DbgFlags::HUMANIZE`] it prints:

```text
size:     815
capacity: 1215

2_407 B 100.00% ⏺: example::Struct<example::TestEnum, example::Data<alloc::vec::Vec<u8>>>
   16 B   0.66% ├╴a: example::TestEnum
                │ ├╴Variant: Unnamed
    8 B   0.33% │ ├╴0: usize
    1 B   0.04% │ ╰╴1: u8
1_183 B  49.15% ├╴b: example::Data<alloc::vec::Vec<u8>>
  724 B  30.08% │ ├╴a: alloc::vec::Vec<u8>
  424 B  17.62% │ ├╴b: alloc::vec::Vec<i32>
   35 B   1.45% │ ╰╴c: (usize, alloc::string::String)
    8 B   0.33% │   ├╴0: usize
   27 B   1.12% │   ╰╴1: alloc::string::String
    8 B   0.33% ├╴test: isize
1_200 B  49.85% ╰╴s: std::collections::hash::set::HashSet<usize>
```

If we use [`DbgFlags::empty()`] it prints:

```text
size:     815
capacity: 1215

985 B ⏺
 16 B ├╴a
      │ ├╴Variant: Unnamed
  8 B │ ├╴0
  1 B │ ╰╴1
823 B ├╴b
724 B │ ├╴a
 64 B │ ├╴b
 35 B │ ╰╴c
  8 B │   ├╴0
 27 B │   ╰╴1
  8 B ├╴test
138 B ╰╴s
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

* `BTreeMap`, and `BTreeSet`, are not currently supported as we still
  have to figure out a way to precisely measure their memory size and capacity.

[`MemDbg`]: https://docs.rs/mem_dbg/latest/mem_dbg/trait.MemDbg.html
[`MemSize`]: https://docs.rs/mem_dbg/latest/mem_dbg/trait.MemSize.html
[`std::mem::size_of`]: https://doc.rust-lang.org/std/mem/fn.size_of.html
[`DbgFlags::CAPACITY`]: https://docs.rs/mem_dbg/latest/mem_dbg/struct.DbgFlags.html#associatedconstant.CAPACITY
[`DbgFlags::HUMANIZE`]: https://docs.rs/mem_dbg/latest/mem_dbg/struct.DbgFlags.html#associatedconstant.HUMANIZE
[`DbgFlags::empty()`]: https://docs.rs/mem_dbg/latest/mem_dbg/struct.DbgFlags.html#method.empty
[`CopyType`]: https://docs.rs/mem_dbg/latest/mem_dbg/trait.CopyType.html
[`cap`]: (https:/crates.io/crates/cap)
[`get-size`]: (https://crates.io/crates/get_size)
[`deepsize`]: (https://crates.io/crates/deepsize)
[`size-of`]: (https://crates.io/crates/size_of)