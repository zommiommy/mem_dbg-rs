# mem_dbg

[![downloads](https://img.shields.io/crates/d/mem_dbg)](https://crates.io/crates/mem_dbg)
[![dependents](https://img.shields.io/librariesio/dependents/cargo/mem_dbg)](https://crates.io/crates/mem_dbg/reverse_dependencies)
![GitHub CI](https://github.com/zommiommy/mem_dbg-rs/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/mem_dbg)
[![](https://tokei.rs/b1/github/zommiommy/mem_dbg-rs?type=Rust,Python)](https://github.com/zommiommy/mem_dbg-rs)
[![Latest version](https://img.shields.io/crates/v/mem_dbg.svg)](https://crates.io/crates/mem_dbg)
[![Documentation](https://docs.rs/mem_dbg/badge.svg)](https://docs.rs/mem_dbg)
[![Coverage Status](https://coveralls.io/repos/github/zommiommy/mem_dbg-rs/badge.svg?branch=main)](https://coveralls.io/github/zommiommy/mem_dbg-rs?branch=main)

Traits and associated procedural macros to inspect recursively the memory usage
and layout of a value.

The trait [`MemSize`] can be used to compute the overall memory usage of a value
in bytes; the standard library function [`std::mem::size_of`] returns the
*stack* size of a type in bytes, but it does not take into consideration heap
memory. We provide implementations for most basic types, a derive macro for
structs and enums whose fields implement [`MemSize`], and support for a few other
crates.

The trait [`MemDbg`], which depends on [`MemSize`], can be used to display the
recursive layout of a value, together with the size of each part and the
associated padding bytes. Also in this case we provide implementations for most
basic types, a derive macro for structs and enums whose fields implement
[`MemDbg`], and support for a few other crates.

## Why `MemSize`

Other traits partially provide the functionality of [`MemSize`], but either they
require implementing manually a trait, which is prone to error, or they do not
provide the flexibility necessary for [`MemDbg`]. Most importantly, [`MemSize`]
uses the type system to avoid iterating over the content of a container (a
vector, etc.) when it is not necessary, making it possible to compute instantly
the size of values occupying hundreds of gigabytes of heap memory.

This is the result of the benchmark `bench_hash_map` contained in the `examples`
directory. It builds a hash map with a hundred million entries and then measures
its heap size:

```test
Allocated:    2281701509
get_size:     1879048240 152477833 ns
deep_size_of: 1879048240 152482000 ns
size_of:      2281701432 152261958 ns
mem_size:     2281701424 209 ns
```

The first line is the number of bytes allocated by the program as returned by
[`cap`]. Then, we display the result of [`get-size`], [`deepsize`], [`size-of`],
and our own [`MemSize`]. Note that the first two crates are just measuring the
space used by the items, and not by the data structure (i.e., they are not
taking into account the load factor and the power-of-two size constraint of the
hash map). Moreover, all other crates are about six orders of magnitude slower
than our implementation, due to the necessity to iterate over all elements.

## Padding

The trait [`MemDbg`] is useful to display the layout of a value and understand
how much memory is used by each part. In particular, it exploits the new stable
macro [`std::mem::offset_of`] to display the padding of each field in square
brackets; moreover, the flag [`DbgFlags::RUST_LAYOUT`] makes it possible to
display structures in the layout used by the Rust compiler, rather than
that given by declaration order.

These features are also available for enums using the feature `offset_of_enum`,
which however needs the nightly compiler, as it enables the unstable feature
`offset_of_enum`.

## Features

- `offset_of_enum`: support for padding and for the `DbgFlags::RUST_LAYOUT` flag
  for enums. Requires the nightly compiler as it enables the unstable feature
  `offset_of_enum`. Calling `mem_dbg` with the flag `DbgFlags::RUST_LAYOUT`
  without this feature enabled will result in a panic.
  
- `half`: support for the [`half`] crate.

- `maligned`: support for the [`maligned`] crate.

- `mmap-rs`: support for the [`mmap-rs`] crate.

- `rand`: support for the [`rand`] crate.

## Examples

This is an example program using [`MemSize`] and [`MemDbg`]. Note that we cannot
visualize the effect of the useful [`DbgFlags::COLOR`] flag, which colorizes
sizes depending on their magnitude.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {

# #[cfg(all(feature = "std", feature = "derive"))]
# #[cfg_attr(feature = "offset_of_enum", feature(offset_of_enum))]
# {
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
    c: (u8, String),
}

#[derive(MemSize, MemDbg)]
union SingletonUnion<A: Copy> {
    a: A
}

#[derive(MemSize, MemDbg)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Union(SingletonUnion<u8>),
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
println!();

s.mem_dbg(DbgFlags::empty())?;

println!();

println!("size:     {}", s.mem_size(SizeFlags::default()));
println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));
println!();

s.mem_dbg(DbgFlags::default() | DbgFlags::CAPACITY | DbgFlags::HUMANIZE)?;

#[cfg(feature = "offset_of_enum")]
{
    println!();

    println!("size:     {}", s.mem_size(SizeFlags::default()));
    println!("capacity: {}", s.mem_size(SizeFlags::CAPACITY));
    println!();

    s.mem_dbg(DbgFlags::empty() | DbgFlags::RUST_LAYOUT)?;
}
# }
# Ok(())
# }
```

The previous program prints:

```test
size:     807
capacity: 1207

807 B ⏺
 16 B ├╴a
      │ ├╴Variant: Unnamed
  8 B │ ├╴0
  1 B │ ╰╴1
783 B ├╴b
724 B │ ├╴a
 24 B │ ├╴b
 35 B │ ╰╴c
  1 B │   ├╴0 [7B]
 27 B │   ╰╴1
  8 B ╰╴test

size:     807
capacity: 1207

1.207 kB 100.00% ⏺: readme::main::Struct<readme::main::TestEnum, readme::main::Data<alloc::vec::Vec<u8>>>
   16  B   1.33% ├╴a: readme::main::TestEnum
                 │ ├╴Variant: Unnamed
    8  B   0.66% │ ├╴0: usize
    1  B   0.08% │ ╰╴1: u8
1.183 kB  98.01% ├╴b: readme::main::Data<alloc::vec::Vec<u8>>
  724  B  59.98% │ ├╴a: alloc::vec::Vec<u8>
  424  B  35.13% │ ├╴b: alloc::vec::Vec<i32>
   35  B   2.90% │ ╰╴c: (u8, alloc::string::String)
    1  B   0.08% │   ├╴0: u8 [7B]
   27  B   2.24% │   ╰╴1: alloc::string::String
    8  B   0.66% ╰╴test: isize
```

If run with the feature `offset_of_enum`, it prints:

```text
size:     807
capacity: 1207

807 B ⏺
 16 B ├╴a
      │ ├╴Variant: Unnamed
  8 B │ ├╴0
  1 B │ ╰╴1 [6B]
783 B ├╴b
724 B │ ├╴a
 24 B │ ├╴b
 35 B │ ╰╴c
  1 B │   ├╴0 [7B]
 27 B │   ╰╴1
  8 B ╰╴test

size:     807
capacity: 1207

1.207 kB 100.00% ⏺: readme::main::Struct<readme::main::TestEnum, readme::main::Data<alloc::vec::Vec<u8>>>
   16  B   1.33% ├╴a: readme::main::TestEnum
                 │ ├╴Variant: Unnamed
    8  B   0.66% │ ├╴0: usize
    1  B   0.08% │ ╰╴1: u8 [6B]
1.183 kB  98.01% ├╴b: readme::main::Data<alloc::vec::Vec<u8>>
  724  B  59.98% │ ├╴a: alloc::vec::Vec<u8>
  424  B  35.13% │ ├╴b: alloc::vec::Vec<i32>
   35  B   2.90% │ ╰╴c: (u8, alloc::string::String)
    1  B   0.08% │   ├╴0: u8 [7B]
   27  B   2.24% │   ╰╴1: alloc::string::String
    8  B   0.66% ╰╴test: isize

size:     807
capacity: 1207

807 B ⏺
783 B ├╴b
724 B │ ├╴a
 24 B │ ├╴b
 35 B │ ╰╴c
  1 B │   ├╴0 [7B]
 27 B │   ╰╴1
 16 B ├╴a
      │ ├╴Variant: Unnamed
  1 B │ ├╴1 [6B]
  8 B │ ╰╴0
  8 B ╰╴test
```

## Caveats

- We support out-of-the-box most basic types, and tuples up to size ten. The
  derive macros `MemSize`/`MemDbg` will generate implementations for structs and
  enums whose fields implement the associated interface: if this is not the case
  (e.g., because of the orphan rule) one can implement the traits manually.

- If you invoke the methods of this crate on a shared reference, the compiler
  will automatically dereference it, and the method will be invoked on the
  referenced type:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use mem_dbg::*;

let mut x: [i32; 4] = [0, 0, 0, 0];

assert_eq!(
    (&x).mem_size(SizeFlags::default()),
    std::mem::size_of::<[i32; 4]>()
);

assert_eq!(
    (&mut x).mem_size(SizeFlags::default()),
    std::mem::size_of::<&mut [i32; 4]>()
);

assert_eq!(
    <&[i32; 4] as MemSize>::mem_size(&&x, SizeFlags::default()),
    std::mem::size_of::<&[i32; 4]>()
);
# Ok(())
# }
```

- Computation of the size of arrays, slices, and vectors will be performed by
  iterating over their elements unless the type is a copy type that does not
  contain non-`'static` references and it is declared as such using the attribute
  `#[copy_type]`. See [`CopyType`] for more details.

- The content of vectors and slices is not expanded recursively as the output
  might be too complex; this might change in the future (e.g., via a flag)
  should interesting use cases arise.

- `BTreeMap`/`BTreeSet` are not currently supported as we still have to
  figure out a way to precisely measure their memory size and capacity.

- Regarding `union`s, we only support completely the special case of the single
  field `union`, for which we implement both the derive macros `MemSize`/`MemDbg`.
  For the more complex cases of unions with multiple fields, we only provide the
  `MemSize` derive macro with partial support, excluding support for the
  `SizeFlags::FOLLOW_REFS` flag. If full support for derive macros `MemSize`/`MemDbg`
  in the case of an union with multiple fields, one can implement the traits manually.
  
[`MemDbg`]: <https://docs.rs/mem_dbg/latest/mem_dbg/trait.MemDbg.html>
[`MemSize`]: <https://docs.rs/mem_dbg/latest/mem_dbg/trait.MemSize.html>
[`std::mem::size_of`]: <https://doc.rust-lang.org/std/mem/fn.size_of.html>
[`DbgFlags::RUST_LAYOUT`]: <https://docs.rs/mem_dbg/latest/mem_dbg/struct.DbgFlags.html#associatedconstant.RUST_LAYOUT>
[`DbgFlags::COLOR`]: <https://docs.rs/mem_dbg/latest/mem_dbg/struct.DbgFlags.html#associatedconstant.COLOR>
[`CopyType`]: <https://docs.rs/mem_dbg/latest/mem_dbg/trait.CopyType.html>
[`cap`]: <https:/crates.io/crates/cap>
[`get-size`]: <https://crates.io/crates/get_size>
[`deepsize`]: <https://crates.io/crates/deepsize>
[`size-of`]: <https://crates.io/crates/size_of>
[`maligned`]: <https://crates.io/crates/maligned>
[`mmap-rs`]: <https://crates.io/crates/mmap-rs>
[`half`]: <https://crates.io/crates/half>
[`rand`]: <https://crates.io/crates/rand>
