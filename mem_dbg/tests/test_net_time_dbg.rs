/*
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Tests that network and time types work with both `MemSize` and
//! `MemDbg`, including inside derived structures.

#![cfg(feature = "derive")]

use anyhow::Result;
use core::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use core::time::Duration;
use mem_dbg::{DbgFlags, MemDbg, MemSize, SizeFlags};

#[derive(MemSize, MemDbg)]
#[mem_size(flat)]
struct NetTime {
    addr: IpAddr,
    socket: SocketAddr,
    duration: Duration,
}

#[test]
fn test_net_time_derive() -> Result<()> {
    let value = NetTime {
        addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
        socket: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80)),
        duration: Duration::from_secs(1),
    };

    assert_eq!(
        value.mem_size(SizeFlags::default()),
        core::mem::size_of::<NetTime>()
    );

    let mut output = String::new();
    value.mem_dbg_on(&mut output, DbgFlags::default())?;
    assert!(output.contains("duration"));
    Ok(())
}
