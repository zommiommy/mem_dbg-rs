# Change Log

## [Unreleased]

### Changed

- **Breaking** (size): `mmap_rs::Mmap` and `mmap_rs::MmapMut` now always
  count their mapped region as part of `mem_size`. Previously the bytes were
  only counted under `SizeFlags::FOLLOW_REFS`, which is inconsistent with
  ownership semantics (the mapping is unmapped on drop).

### New

- Added handle-only `MemSize`/`MemDbg` implementations for `*const T`,
  `*mut T`, `std::rc::Weak<T>`, and `std::sync::Weak<T>`. None of these are
  followed: their referents are neither dereferenced nor counted, and the
  `Weak` impls do not bound the target type.

- Added `MemSize`/`MemDbg` implementations for `core::cmp::Reverse`,
  `core::ops::Bound`, `core::task::Poll`, and `core::ops::ControlFlow`.
  These wrappers only visit the payload variant that is actually present.

- Added generic `MemSize`/`MemDbg` implementations for
  `core::pin::Pin<P>`. `Pin<P>` keeps the memory accounting and traversal
  policy of `P`.

- Added `MemSize`/`MemDbg` implementations for `std::sync::OnceLock`. It
  uses the same initialized-value accounting as `core::cell::OnceCell`.

- Added `MemSize`/`MemDbg` implementations for `Cow<'_, B>`. Borrowed
  values follow reference rules, and owned values use `B::Owned` accounting.

- Added `MemSize`/`MemDbg` implementations for `std::collections::BinaryHeap<T>`. Its accounting follows the same Vec-backed element and capacity rules as `Vec<T>`.

### Improved

- Tighter `BTreeMap`/`BTreeSet` heap estimate: replaced the weighted-average
  node-count formula (which silently dropped the root and intermediate internal
  nodes when `len` was just above `CAPACITY`) with a level-by-level walk using
  `FILL = B + 1 = 7` items per node. Matches the `cap` allocator within ~1% on
  a 100M-element `BTreeSet<usize>` and stays inside the `test_correctness`
  bounds at every measured size.

### Fixed

- `core::cell::OnceCell<T>` and `std::sync::OnceLock<T>` `_mem_dbg_rec_on`
  were no-ops: they delegated to `Option::<&T>::_mem_dbg_rec_on` (the
  empty default). Now they unwrap the option and recurse into `T` when
  populated, so `MemDbg` matches `MemSize`.

- `std::sync::Mutex<T>` and `std::sync::RwLock<T>` `MemDbg` now recurse into
  the inner `T` under default flags. Previously dispatch went through the
  `MutexGuard<T>` / `RwLockReadGuard<T>` impl, which is gated on
  `DbgFlags::FOLLOW_REFS` and silently dropped all children under default flags
  - while `MemSize` always recursed.


## [0.4.1] - 2026-03-25

### Fixed

- Fixed a few quirks on 32-bit platforms, in particular B-tree size estimation.

- Fixed panic due to non-ZST being sorted by offset before a ZST in the same
  position.

### Changed

- `mem_size_flat`/`mem_size_rec` attributes are now
  `mem_size(flat)`/`mem_size(rec)`. Warnings are emitted when using the old
  syntax.

### Improved

- More precise collection size estimation depending on the platform.

## [0.4.0] - 2026-02-11

### New

- New check for potentially `#[mem_size_flat]` types; it causes a compile error, and
  it can be silenced using the `#[mem_size_rec]` attribute.

- Added implementations for `Result`.

### Changed

- `CopyType` has become `FlatType`. `#[copy_type]` is now `#[mem_size_flat]`.
  `#[move_type]` is now `#[mem_size_rec]`. The `Copy + 'static` bound enforced
  by the derive macro has been removed. This change makes it possible to use
  optimized code on atomic types, even if they are not `Copy`.

- `FOLLOW_RC` has been renamed `FOLLOW_RCS` for uniformity with `FOLLOW_REFS`.

- The occupancy of references and counted references are now counted once
  when using `FOLLOW_REFS`/`FOLLOW_RCS`.

- Removed no-op `alloc` feature.

- The size of `Rc`/`Arc` does not include anymore the space used by
  `RcInner`/`ArcInner` unless `FOLLOW_RCS` is set.

- Moved to `rand` 0.10.0 and `mmap-rs` 0.7.0.

- Removed single-variant union support. There is now an example
  showing how to handle unions properly with a manual implementation.

### Fixed

- A proper replica of `ArcInner` is now used to measure the occupancy of
  `Arc`.

- Now tuples get the correct `FlatType::Flat` (it used to be always
  `False`).

- Now ranges get the correct `FlatType::Flat` (it used to be always
  `True`).

- Fixed size of `PathBuf` (wasn't considering the buffer).

- Fixed size of `str` (there was a spurious `usize` counted).

- Owned `OsStr` instances now correctly report their size.

- Fixed potential panic when traversing a `RefCell`.

- The `FlatType` of a `maligned::Aligned` type is now correctly
  taken from the wrapped type.

- `BufReader` and `BufWriter` now report the size of their internal buffer.

- Fixed possible identifier clash in derive code for enums.

## [0.3.4] - 2026-01-20

### New

- Support for `VecDeque`, `BTreeMap` and `BTreeSet`.

- Vastly improved test coverage, added correctness tests for `mem_size` using
  `cap`, and added output test for `mem_dbg` using `insta`.

### Fixed

- Sets and maps now have a `CopyType`.

- Fixed several old bugs in implementations and output formatting.

### Changed

- Improved `HashMap` and `HashSet` size approximation.

## [0.3.3] - 2026-01-04

### New

- Support for `std::rc::Rc`.

- New `FOLLOW_RC` flag for following reference-counted pointers.

### Fixed

- Completed support for `std::hash::DefaultHasher`.

- Extensive tests uncovered a series of bugs (in particular, in `PathBuf` and
  `OnceCell`) that have been fixed.

- The `std` feature now works as expected. The `alloc` feature is no longer
  supported.

## [0.3.2] - 2025-11-03

### New

- Added support for `std::hash::DefaultHasher`.

## [0.3.1] - 2025-10-16

### Changed

- Switched to the 2024 edition.

- Updated all dependencies.

## [0.3.0] - 2025-02-09

### Fixed

- The `offset_of_nested` feature has been stabilized, so the references to it
  have been removed.

- The check for the `offset_of_enum` feature was partly in the generated
  code, which was problematic as it had to be set from the user crate
  by adding an `offset_of_enum` feature. Now all checks are in the
  code of the derive macro.

## [0.2.5] - 2025-02-06

### New

- `COLOR` flag for colorized output.

- We now print to standard error.

## [0.2.4] - 2024-08-09

### Fixed

- The size of slices was off by 8 bytes.

## [0.2.3] - 2024-08-09

### Fixed

- Mutable and non-mutable slices were returning different sizes within
  structures. At the top level, this is however unavoidable due to
  different auto-dereferencing policies in the compiler, and this behavior
  has been documented. The same problem was affecting `MemDbg`.

## [0.2.2] - 2024-06-03

### Fixed

- Fixed a bug where the compiler was issuing the error `usize cannot be
dereferenced`. Invoking `id_sizes.iter()` in the generated code was
  returning sometimes `&(usize, usize)` and sometimes `(usize, usize)`.
  Using `into_iter` gives us a consistent behavior.

## [0.2.1] - 2024-05-28

### New

- Support for network and time structures.

## [0.2.0] - 2024-04-10

### New

- Support for displaying padding in structures and tuples using the new
  stable macro `offset_of`, with an optional feature `offset_of_enum` for
  enums (the latter requires nightly).

- `MemDbgFlags::RUST_LAYOUT` flag displays structure in compiler layout;
  needs feature `offset_of_enum` for enums (the latter requires nightly).

- Support for other crates such as `maligned` and `rand`.

### Fixed

- Fixed cfgs for mmap-rs because it could not be enabled.

- Fixed size for tuples (it was not considering padding).

## [0.1.8] - 2024-04-07

### Improved

- Fixed cfgs for mmap-rs because it could not be enabled.

## [0.1.7] - 2024-04-07

### Improved

- Added missing CopyTypes for maligned types.

## [0.1.6] - 2024-04-07

### Improved

- Many new implementations for standard types such as RefCell.

## [0.1.4] - 2024-03-18

### Changed

- Removed `.unwrap()` from the examples.
