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
//! Fanout benchmark: 1 producer → N subscribers, slowest-consumer p95.
//!
//! Spawns `--n` subscriber threads and one publisher, then sends `--iters`
//! 1080p RGBA8 DMA-BUF frames.  Each subscriber records per-frame receive
//! latency; the worst p95 across all subscribers is reported.
//!
//! # Usage
//!
//! ```text
//! dmabuf-bench fanout [--n N] [--iters N]
//! ```
//!
//! Default: 3 subscribers, 1 000 iterations.

pub const DEFAULT_N: usize = 3;
pub const DEFAULT_ITERS: usize = 1_000;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Meta {
    size: u64,
    seq: u64,
}
// Safety: Meta is repr(C), Copy, with no padding bytes of undefined value.
unsafe impl iceoryx2::prelude::ZeroCopySend for Meta {}

fn run_fanout_inner(n: usize, iters: usize) -> Result<(), Box<dyn core::error::Error>> {
    use crate::bench_latency::PAYLOAD_BYTES;
    use iceoryx2::service::ipc;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    let svc = format!("bench/dmabuf/fanout/n{n}");
    let barrier = Arc::new(Barrier::new(n + 1));

    // Spawn N subscriber threads; each records per-frame latency.
    let mut handles = Vec::with_capacity(n);
    let mut all_samples: Vec<Arc<std::sync::Mutex<Vec<Duration>>>> = Vec::with_capacity(n);

    for _ in 0..n {
        let svc_clone = svc.clone();
        let bar = Arc::clone(&barrier);
        let samples_cell: Arc<std::sync::Mutex<Vec<Duration>>> =
            Arc::new(std::sync::Mutex::new(Vec::with_capacity(iters)));
        let samples_clone = Arc::clone(&samples_cell);
        all_samples.push(samples_cell);

        let handle = std::thread::spawn(
            move || -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
                let mut sub_ = FdSidecarSubscriber::<ipc::Service, Meta>::create(&svc_clone)?;
                bar.wait();
                for _ in 0..iters {
                    let t0 = Instant::now();
                    while sub_.recv()?.is_none() {}
                    let elapsed = t0.elapsed();
                    if let Ok(mut v) = samples_clone.lock() {
                        v.push(elapsed);
                    }
                }
                Ok(())
            },
        );
        handles.push(handle);
    }

    // Publisher: create after subscribers so open_or_create sees them.
    let mut pub_ = FdSidecarPublisher::<ipc::Service, Meta>::create(&svc)?;
    let zeroes = vec![0u8; PAYLOAD_BYTES as usize];

    barrier.wait(); // release subscribers

    for seq in 0..iters as u64 {
        let fd = memfd_create("bench-fanout-frame", MemfdFlags::CLOEXEC)?;
        rustix::io::write(&fd, &zeroes)?;
        pub_.send(
            Meta {
                size: PAYLOAD_BYTES,
                seq,
            },
            fd,
        )?;
    }

    // Collect results.
    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(format!("subscriber thread error: {e}").into()),
            Err(_) => return Err("subscriber thread panicked".into()),
        }
    }

    // Find slowest-consumer p95.
    let mut slowest_p95 = Duration::ZERO;
    for cell in &all_samples {
        if let Ok(mut v) = cell.lock() {
            v.sort_unstable();
            if v.len() >= 20 {
                let p95 = v[(v.len() * 95) / 100];
                if p95 > slowest_p95 {
                    slowest_p95 = p95;
                }
            }
        }
    }

    println!(
        "dmabuf_fanout_latency subscribers={n} iters={iters} slowest_p95_us={}",
        slowest_p95.as_micros()
    );
    Ok(())
}

pub fn run_fanout() {
    let args: Vec<String> = std::env::args().collect();
    let n = match crate::args::parse_flag(&args, "--n") {
        Some(v) => v,
        None => DEFAULT_N,
    };
    let iters = match crate::args::parse_flag(&args, "--iters") {
        Some(v) => v,
        None => DEFAULT_ITERS,
    };
    if let Err(e) = run_fanout_inner(n, iters) {
        eprintln!("fanout bench error: {e}");
        std::process::exit(1);
    }
}
