# Change Log

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
