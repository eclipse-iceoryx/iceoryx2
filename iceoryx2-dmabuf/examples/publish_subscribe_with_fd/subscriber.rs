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
//! Example: DMA-BUF subscriber — receives frames from a `FdSidecarPublisher` and
//! prints metadata for each received frame.
//!
//! # Run
//!
//! ```text
//! cargo run --example dmabuf-subscriber --features memfd
//! ```
//!
//! Start this subscriber first, then the publisher, or run them concurrently
//! in two terminals.

use iceoryx2::service::ipc;
use iceoryx2_dmabuf::FdSidecarSubscriber;

/// Shared frame metadata (must match `publisher.rs`).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct FrameMeta {
    width: u32,
    height: u32,
    /// FourCC code.
    fourcc: u32,
    /// Monotonically increasing frame sequence number.
    seq: u64,
}
// Safety: FrameMeta is `repr(C)`, `Copy`, and contains no padding bytes of
// undefined value.
unsafe impl iceoryx2::prelude::ZeroCopySend for FrameMeta {}

const SVC: &str = "mos4/frame-plane/example/0";
const EXPECTED_FRAMES: u64 = 100;

fn run() -> Result<(), Box<dyn core::error::Error>> {
    let mut subscriber = FdSidecarSubscriber::<ipc::Service, FrameMeta>::create(SVC)?;

    let mut received = 0u64;
    while received < EXPECTED_FRAMES {
        match subscriber.recv()? {
            Some((meta, _fd)) => {
                println!(
                    "received seq={} {}x{} fourcc={:#010x}",
                    meta.seq, meta.width, meta.height, meta.fourcc
                );
                received += 1;
            }
            None => {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    }
    println!("subscriber done ({received} frames)");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("subscriber error: {e}");
        std::process::exit(1);
    }
}
