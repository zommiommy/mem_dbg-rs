/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Tests that `Option` and `Result` render their active payload as a
//! labeled child, consistently with `ControlFlow`, `Poll`, and `Bound`.

use anyhow::Result;
use mem_dbg::{DbgFlags, MemDbg};

#[test]
fn test_option_some_renders_child() -> Result<()> {
    let value: Option<u32> = Some(7);
    let mut output = String::new();
    value.mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_eq!(output, "8 B ⏺\n4 B ╰╴Some\n");
    Ok(())
}

#[test]
fn test_option_none_renders_no_child() -> Result<()> {
    let value: Option<u32> = None;
    let mut output = String::new();
    value.mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_eq!(output, "8 B ⏺\n");
    Ok(())
}

#[test]
fn test_result_ok_renders_child() -> Result<()> {
    let value: core::result::Result<u32, u8> = Ok(7);
    let mut output = String::new();
    value.mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_eq!(output, "8 B ⏺\n4 B ╰╴Ok\n");
    Ok(())
}

#[test]
fn test_result_err_renders_child() -> Result<()> {
    let value: core::result::Result<u32, u8> = Err(3);
    let mut output = String::new();
    value.mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_eq!(output, "8 B ⏺\n1 B ╰╴Err\n");
    Ok(())
}

#[test]
fn test_nested_option_renders_chain() -> Result<()> {
    let value: Option<Option<u32>> = Some(Some(7));
    let mut output = String::new();
    value.mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_eq!(output, "8 B ⏺\n8 B ╰╴Some\n4 B   ╰╴Some\n");
    Ok(())
}
