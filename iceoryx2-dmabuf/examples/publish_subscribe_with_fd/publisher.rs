// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//
//! Example: DMA-BUF publisher — creates a memfd for each frame and sends it
//! alongside metadata via `FdSidecarPublisher`.
//!
//! # Run (Linux only — requires `memfd_create`)
//!
//! ```text
//! cargo run --example dmabuf-publisher --features memfd
//! ```
//!
//! Start the subscriber first, then the publisher, or run them concurrently
//! in two terminals.

use iceoryx2::service::ipc;
use iceoryx2_dmabuf::FdSidecarPublisher;

/// Shared frame metadata (must match `subscriber.rs`).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct FrameMeta {
    width: u32,
    height: u32,
    /// FourCC code: `0x3231_5659` = `YV12`, `0x3231_5242` = `RG24`, etc.
    fourcc: u32,
    /// Monotonically increasing frame sequence number.
    seq: u64,
}
// Safety: FrameMeta is `repr(C)`, `Copy`, and contains no padding bytes of
// undefined value.
unsafe impl iceoryx2::prelude::ZeroCopySend for FrameMeta {}

const SVC: &str = "mos4/frame-plane/example/0";
const FRAMES: u64 = 100;

fn run() -> Result<(), Box<dyn core::error::Error>> {
    let mut publisher = FdSidecarPublisher::<ipc::Service, FrameMeta>::create(SVC)?;

    for seq in 0..FRAMES {
        let fd = open_frame_fd()?;
        let meta = FrameMeta {
            width: 1920,
            height: 1080,
            fourcc: 0x3231_5659, // YV12
            seq,
        };
        publisher.send(meta, fd)?;
        println!("published frame seq={seq}");
        std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 fps
    }
    println!("publisher done ({FRAMES} frames)");
    Ok(())
}

/// Allocate a frame fd: `memfd_create` on Linux, `/dev/null` on other platforms.
#[cfg(target_os = "linux")]
fn open_frame_fd() -> Result<std::os::fd::OwnedFd, Box<dyn core::error::Error>> {
    use iceoryx2_dmabuf::FdSidecarError;
    use rustix::fs::{MemfdFlags, memfd_create};
    let fd = memfd_create("example-frame", MemfdFlags::CLOEXEC)
        .map_err(|e| FdSidecarError::SideChannelIo(std::io::Error::from(e)))?;
    Ok(fd)
}

/// Stub for non-Linux: returns a /dev/null fd so the example compiles everywhere.
#[cfg(not(target_os = "linux"))]
fn open_frame_fd() -> Result<std::os::fd::OwnedFd, Box<dyn core::error::Error>> {
    use std::os::fd::{FromRawFd, IntoRawFd};
    let f = std::fs::OpenOptions::new().read(true).open("/dev/null")?;
    let raw = f.into_raw_fd();
    // Safety: raw is a valid, open file descriptor we just obtained.
    Ok(unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("publisher error: {e}");
        std::process::exit(1);
    }
}
