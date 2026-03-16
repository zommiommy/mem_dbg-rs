# mem_dbg-derive

[![crates.io](https://img.shields.io/crates/v/mem_dbg-derive.svg)](https://crates.io/crates/mem_dbg-derive)
[![docs.rs](https://docs.rs/mem_dbg-derive/badge.svg)](https://docs.rs/mem_dbg-derive)
[![rustc](https://img.shields.io/badge/rustc-1.85+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![CI](https://github.com/zommiommy/mem_dbg-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/zommiommy/mem_dbg-rs/actions)
![license](https://img.shields.io/crates/l/mem_dbg-derive)
[![downloads](https://img.shields.io/crates/d/mem_dbg-derive)](https://crates.io/crates/mem_dbg-derive)

Procedural derive macros for the [`mem_dbg`](https://crates.io/crates/mem_dbg) crate.

This crate provides the `#[derive(MemSize)]` and `#[derive(MemDbg)]` macros.
You should not depend on this crate directly; instead, use the `mem_dbg`
crate with the `derive` feature (enabled by default).

See the [`mem_dbg` documentation](https://docs.rs/mem_dbg) for usage and examples.
