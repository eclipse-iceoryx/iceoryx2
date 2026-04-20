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

//! Helper binary for the `it_crash_midsend` integration test.
//!
//! Publishes a single frame.  When `DMABUF_CRASH_PHASE=mid-iceoryx2` is set,
//! the publisher calls `raise(SIGSTOP)` between the sidecar send and the
//! iceoryx2 publish, simulating a producer crash mid-send.
//!
//! Linux-only binary.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("fd_sidecar_crash_pub: Linux only");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(5);
    }
}

#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn core::error::Error>> {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::FdSidecarPublisher;
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::io::Write as _;

    const PAYLOAD_LEN: usize = 64;
    const PAYLOAD_BYTE: u8 = 0xAB;
    const CONNECT_SETTLE_MS: u64 = 100;
    const KEEP_ALIVE_MS: u64 = 500;

    let service =
        std::env::var("DMABUF_SERVICE").unwrap_or_else(|_| "crash-midsend-test".to_owned());

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        size: u64,
    }

    let fd = memfd_create("crash-pub-frame", MemfdFlags::CLOEXEC)?;
    {
        use std::os::fd::{AsFd as _, AsRawFd as _, FromRawFd as _};
        let raw = fd.as_fd().as_raw_fd();
        // SAFETY: fd is valid and owned; ManuallyDrop prevents double-close.
        let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(raw) });
        let payload = vec![PAYLOAD_BYTE; PAYLOAD_LEN];
        file.write_all(&payload)?;
    }

    let mut pub_ = FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(&service)?;

    // Give subscriber(s) time to connect.
    std::thread::sleep(std::time::Duration::from_millis(CONNECT_SETTLE_MS));

    // send() calls pause_hook_if_requested() between sidecar and iceoryx2.
    pub_.send(
        Meta {
            size: PAYLOAD_LEN as u64,
        },
        fd,
    )?;

    // Keep alive for the test to observe state.
    std::thread::sleep(std::time::Duration::from_millis(KEEP_ALIVE_MS));

    Ok(())
}
