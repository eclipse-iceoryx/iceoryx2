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

//! Helper binary for the `it_fd_identity` integration test.
//!
//! Run with env var `DMABUF_ROLE=pub` to act as publisher, or
//! `DMABUF_ROLE=sub` to act as subscriber.
//!
//! This binary is Linux-only; it exits with code 1 on other platforms.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("fd_sidecar_fd_identity: Linux only");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() {
    const DEFAULT_SERVICE: &str = "fd-identity-test";

    let role = std::env::var("DMABUF_ROLE").ok().unwrap_or_default();
    let service = std::env::var("DMABUF_SERVICE")
        .ok()
        .unwrap_or_else(|| DEFAULT_SERVICE.to_owned());

    let result = match role.as_str() {
        "pub" => run_publisher(&service),
        "sub" => run_subscriber(&service),
        other => {
            eprintln!("unknown DMABUF_ROLE={other:?}; expected 'pub' or 'sub'");
            std::process::exit(2);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(5);
    }
}

#[cfg(target_os = "linux")]
use iceoryx2::prelude::ZeroCopySend;

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[repr(C)]
struct Meta {
    size: u64,
}

#[cfg(target_os = "linux")]
fn run_publisher(service: &str) -> Result<(), Box<dyn core::error::Error>> {
    use iceoryx2_dmabuf::FdSidecarPublisher;
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::io::Write as _;

    const PAYLOAD_BYTE: u8 = 0xAB;
    const PAYLOAD_LEN: usize = 64;
    const CONNECT_SETTLE_MS: u64 = 100;
    const RECV_WINDOW_MS: u64 = 300;

    let fd = memfd_create("fd-identity-pub", MemfdFlags::CLOEXEC)?;

    // Write magic bytes via a borrowed File view.
    {
        use std::os::fd::{AsFd as _, AsRawFd as _, FromRawFd as _};
        let raw = fd.as_fd().as_raw_fd();
        // SAFETY: fd is valid and owned; ManuallyDrop prevents double-close.
        let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(raw) });
        let payload = vec![PAYLOAD_BYTE; PAYLOAD_LEN];
        file.write_all(&payload)?;
    }

    // Obtain (st_dev, st_ino) before publishing.
    let stat = rustix::fs::fstat(&fd)?;
    let st_dev = stat.st_dev;
    let st_ino = stat.st_ino;

    // Print to stdout so the parent test can parse it.
    println!("PUB_STAT:{st_dev}:{st_ino}");
    std::io::stdout().flush()?;

    let mut pub_ = FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(service)?;

    // Give subscriber time to connect.
    std::thread::sleep(std::time::Duration::from_millis(CONNECT_SETTLE_MS));

    pub_.send(
        Meta {
            size: PAYLOAD_LEN as u64,
        },
        fd,
    )?;

    // Keep publisher alive long enough for subscriber to recv.
    std::thread::sleep(std::time::Duration::from_millis(RECV_WINDOW_MS));

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_subscriber(service: &str) -> Result<(), Box<dyn core::error::Error>> {
    use iceoryx2_dmabuf::FdSidecarSubscriber;
    use std::io::Write as _;

    const POLL_INTERVAL_MS: u64 = 10;
    const TIMEOUT_SECS: u64 = 5;

    let mut sub_ = FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(service)?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(TIMEOUT_SECS);
    let mut received = false;

    while std::time::Instant::now() < deadline {
        match sub_.recv()? {
            Some((_meta, fd)) => {
                let stat = rustix::fs::fstat(&fd)?;
                println!("SUB_STAT:{}:{}", stat.st_dev, stat.st_ino);
                std::io::stdout().flush()?;
                received = true;
                break;
            }
            None => {
                std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS));
            }
        }
    }

    if !received {
        return Err("subscriber: timeout waiting for frame".into());
    }
    Ok(())
}
