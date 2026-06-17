/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Tests that zero-sized tuple fields are not attributed spurious padding,
//! whatever offset the compiler assigns them.

use anyhow::Result;
use mem_dbg::{DbgFlags, MemDbg};

fn assert_no_padding(output: &str) -> Result<()> {
    assert!(
        !output.contains('['),
        "spurious padding annotation:\n{output}"
    );
    Ok(())
}

#[test]
fn test_zst_tuples_show_no_padding() -> Result<()> {
    // There is no real padding in any of these tuples, so no `[..B]`
    // annotation must appear, wherever the compiler places the ZSTs.
    let mut output = String::new();
    (1_u64, ()).mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_no_padding(&output)?;

    output.clear();
    ((), 1_u64).mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_no_padding(&output)?;

    output.clear();
    ((), (), 1_u64).mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_no_padding(&output)?;

    output.clear();
    (1_u32, (), 2_u32, ()).mem_dbg_on(&mut output, DbgFlags::empty())?;
    assert_no_padding(&output)?;
    Ok(())
}
