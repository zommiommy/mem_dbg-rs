# mem_dbg-derive

[![downloads](https://img.shields.io/crates/d/mem_dbg)](https://crates.io/crates/mem_dbg)
![license](https://img.shields.io/crates/l/mem_dbg)
[![Latest version](https://img.shields.io/crates/v/mem_dbg-derive.svg)](https://crates.io/crates/mem_dbg-derive)
[![Documentation](https://docs.rs/mem_dbg/badge.svg)](https://docs.rs/mem_dbg)

Procedural derive macros for the [`mem_dbg`](https://crates.io/crates/mem_dbg) crate.

This crate provides the `#[derive(MemSize)]` and `#[derive(MemDbg)]` macros.
You should not depend on this crate directly; instead, use the `mem_dbg`
crate with the `derive` feature (enabled by default).

See the [`mem_dbg` documentation](https://docs.rs/mem_dbg) for usage and examples.
