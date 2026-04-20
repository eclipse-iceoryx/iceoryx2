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
//! Throughput benchmark: sustained frame rate for 1080p RGBA8 DMA-BUF frames.
//!
//! Sends `--frames` frames as fast as possible (publisher side) and receives
//! them on the subscriber side in the same process.  Reports total wall-clock
//! time and frames-per-second.
//!
//! # Usage
//!
//! ```text
//! dmabuf-bench throughput [--frames N]
//! ```
//!
//! Default: 100 000 frames.

pub const DEFAULT_FRAMES: usize = 100_000;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Meta {
    size: u64,
}
// Safety: Meta is repr(C), Copy, with no padding bytes of undefined value.
unsafe impl iceoryx2::prelude::ZeroCopySend for Meta {}

fn run_throughput_inner(frames: usize) -> Result<(), Box<dyn core::error::Error>> {
    use crate::bench_latency::PAYLOAD_BYTES;
    use iceoryx2::service::ipc;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::time::Instant;

    let svc = "bench/dmabuf/throughput";
    let mut pub_ = FdSidecarPublisher::<ipc::Service, Meta>::create(svc)?;
    let mut sub_ = FdSidecarSubscriber::<ipc::Service, Meta>::create(svc)?;

    let zeroes = vec![0u8; PAYLOAD_BYTES as usize];
    let start = Instant::now();

    for _ in 0..frames {
        let fd = memfd_create("bench-tp-frame", MemfdFlags::CLOEXEC)?;
        rustix::io::write(&fd, &zeroes)?;
        pub_.send(
            Meta {
                size: PAYLOAD_BYTES,
            },
            fd,
        )?;
        while sub_.recv()?.is_none() {}
    }

    let elapsed = start.elapsed();
    let fps = frames as f64 / elapsed.as_secs_f64();
    println!("dmabuf_publish_throughput ({frames} frames)");
    println!(
        "  fps={fps:.1}  total_ms={:.0}",
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(())
}

pub fn run_throughput() {
    let args: Vec<String> = std::env::args().collect();
    let frames = match crate::args::parse_flag(&args, "--frames") {
        Some(v) => v,
        None => DEFAULT_FRAMES,
    };
    if let Err(e) = run_throughput_inner(frames) {
        eprintln!("throughput bench error: {e}");
        std::process::exit(1);
    }
}
