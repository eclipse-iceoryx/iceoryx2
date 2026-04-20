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
//! Latency benchmark: round-trip time for a single 1080p RGBA8 DMA-BUF frame.
//!
//! Measures publisher→subscriber end-to-end latency (send + recv) over
//! `--iters` iterations after `--warmup` warm-up rounds.  Reports p50, p95,
//! and max in microseconds.
//!
//! # Usage
//!
//! ```text
//! dmabuf-bench latency [--iters N] [--warmup N]
//! ```
//!
//! Default: 10 000 iterations, 100 warm-up iterations.

pub const PAYLOAD_BYTES: u64 = 8_294_400; // 1920 x 1080 x 4 (RGBA8)
pub const DEFAULT_WARMUP: usize = 100;
pub const DEFAULT_ITERS: usize = 10_000;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Meta {
    size: u64,
}
// Safety: Meta is repr(C), Copy, with no padding bytes of undefined value.
unsafe impl iceoryx2::prelude::ZeroCopySend for Meta {}

fn run_latency_inner(iters: usize, warmup: usize) -> Result<(), Box<dyn core::error::Error>> {
    use iceoryx2::service::ipc;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::time::Instant;

    let svc = "bench/dmabuf/latency";
    let mut pub_ = FdSidecarPublisher::<ipc::Service, Meta>::create(svc)?;
    let mut sub_ = FdSidecarSubscriber::<ipc::Service, Meta>::create(svc)?;

    let total = warmup + iters;
    let mut samples = Vec::with_capacity(iters);
    let zeroes = vec![0u8; PAYLOAD_BYTES as usize];

    for i in 0..total {
        let fd = memfd_create("bench-frame", MemfdFlags::CLOEXEC)?;
        rustix::io::write(&fd, &zeroes)?;

        let t0 = Instant::now();
        pub_.send(
            Meta {
                size: PAYLOAD_BYTES,
            },
            fd,
        )?;
        while sub_.recv()?.is_none() {}
        let elapsed = t0.elapsed();

        if i >= warmup {
            samples.push(elapsed);
        }
    }

    samples.sort_unstable();
    let p50 = samples[iters / 2];
    let p95 = samples[(iters * 95) / 100];
    let max = match samples.last() {
        Some(v) => *v,
        None => return Err("no samples collected".into()),
    };

    println!("dmabuf_publish_latency 1080p RGBA8 ({iters} iters, {warmup} warmup)");
    println!(
        "  p50={} us  p95={} us  max={} us",
        p50.as_micros(),
        p95.as_micros(),
        max.as_micros()
    );
    Ok(())
}

pub fn run_latency() {
    let args: Vec<String> = std::env::args().collect();
    let iters = match crate::args::parse_flag(&args, "--iters") {
        Some(v) => v,
        None => DEFAULT_ITERS,
    };
    let warmup = match crate::args::parse_flag(&args, "--warmup") {
        Some(v) => v,
        None => DEFAULT_WARMUP,
    };
    if let Err(e) = run_latency_inner(iters, warmup) {
        eprintln!("latency bench error: {e}");
        std::process::exit(1);
    }
}
