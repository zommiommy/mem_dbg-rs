# Change Log

## [0.4.0] - 2026-02-11

### New

* New check for potentially `#[mem_size_flat]` types; it causes a compile error, and
  it can be silenced using the `#[mem_size_rec]` attribute.

* Added implementations for `Result`.

### Changed

* `CopyType` has become `FlatType`. `#[copy_type]` is now `#[mem_size_flat]`.
  `#[move_type]` is now `#[mem_size_rec]`. The `Copy + 'static` bound enforced
  by the derive macro has been removed. This change makes it possible to use
  optimized code on atomic types, even if they are not `Copy`.

* `FOLLOW_RC` has been renamed `FOLLOW_RCS` for uniformity with `FOLLOW_REFS`.

* The occupancy of references and counted references are now counted once
  when using `FOLLOW_REFS`/`FOLLOW_RCS`.

* Removed no-op `alloc` feature.

* The size of `Rc`/`Arc` does not include anymore the space used by
  `RcInner`/`ArcInner` unless `FOLLOW_RCS` is set.

* Moved to `rand` 0.10.0 and `mmap-rs` 0.7.0.

* Removed single-variant union support. There is now an example
  showing how to handle unions properly with a manual implementation.

### Fixed

* A proper replica of `ArcInner` is now used to measure the occupancy of
  `Arc`.

* Now tuples get the correct `FlatType::Flat` (it used to be always
  `False`).

* Now ranges get the correct `FlatType::Flat` (it used to be always
  `True`).

* Fixed size of `PathBuf` (wasn't considering the buffer).

* Fixed size of `str` (there was a spurious `usize` counted).

* Owned `OsStr` instances now correctly report their size.

* Fixed potential panic when traversing a `RefCell`.

* The `FlatType` of a `maligned::Aligned` type is now correctly
  taken from the wrapped type.

* `BufReader` and `BufWriter` now report the size of their internal buffer.

* Fixed possible identifier clash in derive code for enums.

## [0.3.4] - 2026-01-20

### New

* Support for `VecDeque`, `BTreeMap` and `BTreeSet`.

* Vastly improved test coverage, added correctness tests for `mem_size` using
  `cap`, and added output test for `mem_dbg` using `insta`.

### Fixed

* Sets and maps now have a `CopyType`.

* Fixed several old bugs in implementations and output formatting.

### Changed

* Improved `HashMap` and `HashSet` size approximation.

## [0.3.3] - 2026-01-04

### New

* Support for `std::rc::Rc`.

* New `FOLLOW_RC` flag for following reference-counted pointers.

### Fixed

* Completed support for `std::hash::DefaultHasher`.

* Extensive tests uncovered a series of bugs (in particular, in `PathBuf` and
  `OnceCell`) that have been fixed.

* The `std` feature now works as expected. The `alloc` feature is no longer
  supported.

## [0.3.2] 2025-11-03

### New

* Added support for `std::hash::DefaultHasher`.

## [0.3.1] 2025-10-16

### Changed

* Switched to the 2024 edition.

* Updated all dependencies.

## [0.3.0] 2025-02-09

### Fixed

* The `offset_of_nested` feature has been stabilized, so the references to it
  have been removed.

* The check for the `offset_of_enum` feature was partly in the generated
  code, which was problematic as it had to be set from the user crate
  by adding an `offset_of_enum` feature. Now all checks are in the
  code of the derive macro.
  
## [0.2.5] 2025-02-06

### New

* `COLOR` flag for colorized output.

* We now print to standard error.

## [0.2.4] - 2024-08-09

### Fixed

* The size of slices was off by 8 bytes.

## [0.2.3] - 2024-08-09

### Fixed

* Mutable and non-mutable slices were returning different sizes within
  structures. At the top level, this is however unavoidable due to
  different autodeferentiation policies in the compiler, and this behavior
  has been documented. The same problem was affecting `MemDbg`.

## [0.2.2] - 2024-06-03

### Fixed

* Fixed a bug where the compiler was issuing the error `usize cannot be
  dereferenced`. Invoking `id_sizes.iter()` in the generated code was
  returning sometimes `&(usize, usize)` and sometimes `(usize, usize)`.
  Using `into_iter` gives us a consistent behavior.

## [0.2.1] - 2024-05-28

### New

* Support for network and time structures.

## [0.2.0] - 2024-04-10

### New

* Support for displaying padding in structures and tuples using the new
  stable macro `offset_of`, with an optional feature `offset_of_enum` for
  enums (the latter requires nightly).

* `MemDbgFlags::RUST_LAYOUT` flag displays structure in compiler layout;
  needs feature `offset_of_enum` for enums (the latter requires nightly).

* Support for other crates such as `maligned` and `rand`.

### Fixed

* Fixed cfgs for mmap-rs because it could not be enabled.

* Fixed size for tuples (it was not considering padding).

## [0.1.8] - 2024-04-07

### Improved

* Fixed cfgs for mmap-rs because it could not be enabled.

## [0.1.7] - 2024-04-07

### Improved

* Added missing CopyTypes for maligned types.

## [0.1.6] - 2024-04-07

### Improved

* Many new implementations for standard types such as RefCell.

## [0.1.4] - 2024-03-18

### Changed

* Removed `.unwrap()` from the examples.
