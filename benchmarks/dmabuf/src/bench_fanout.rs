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

//! Fanout benchmark: 1 producer → N subscribers, slowest-consumer p95.
//!
//! Spawns `--n` subscriber threads and one publisher in the same process,
//! then sends `--iters` 4 MB memfd DMA-BUF frames using the typed
//! `DmaBufPublisher` / `DmaBufSubscriber` convenience layer.  Each subscriber
//! records per-frame receive latency; the worst p95 across all subscribers is
//! reported.
//!
//! # Usage
//!
//! ```text
//! iceoryx2-benchmarks-dmabuf fanout [--n N] [--iters N]
//! ```
//!
//! Default: 3 subscribers, 1 000 iterations.

#[cfg(target_os = "linux")]
mod linux {
    /// Default subscriber count.
    const DEFAULT_N: usize = 3;
    /// Default number of iterations per subscriber.
    const DEFAULT_ITERS: usize = 1_000;
    /// Maximum spin-poll attempts per frame before treating the receive as stalled.
    const MAX_RECV_POLLS: usize = 1_000_000;
    /// Minimum samples required to compute a meaningful p95 percentile.
    const MIN_SAMPLES_FOR_P95: usize = 20;

    fn run_inner(n: usize, iters: usize) -> Result<(), Box<dyn core::error::Error>> {
        use crate::bench_latency::PAYLOAD_BYTES;
        use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};
        use std::sync::{Arc, Barrier};
        use std::time::{Duration, Instant};

        let svc = format!("bench/dmabuf/fanout/n{n}");

        // Publisher MUST be created first: its UDS socket must be bound before
        // subscribers attempt to connect. The barrier only synchronises the
        // measurement start, not connection setup.
        let mut pub_ = DmaBufPublisher::<u64>::create(&svc)?;

        let barrier = Arc::new(Barrier::new(n + 1));

        // Spawn N subscriber threads; each records per-frame receive latency.
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
                    let mut sub_ = DmaBufSubscriber::<u64>::create(&svc_clone)?;
                    bar.wait(); // wait until all subscribers are connected + publisher is ready
                    for _ in 0..iters {
                        let t0 = Instant::now();
                        let mut polls = 0usize;
                        loop {
                            if sub_.receive()?.is_some() {
                                break;
                            }
                            polls += 1;
                            if polls >= MAX_RECV_POLLS {
                                return Err("subscriber stalled: no sample after max polls".into());
                            }
                        }
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

        barrier.wait(); // release subscriber threads; start measurement

        for seq in 0..iters as u64 {
            let buf = crate::common::make_memfd(PAYLOAD_BYTES)?;
            pub_.publish(seq, &buf)?;
        }

        // Collect thread results.
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
                if v.len() >= MIN_SAMPLES_FOR_P95 {
                    let p95 = v[(v.len() * 95) / 100];
                    if p95 > slowest_p95 {
                        slowest_p95 = p95;
                    }
                }
            }
        }

        println!(
            "fanout_subscribers={n} fanout_iters={iters} fanout_slowest_p95_us={}",
            slowest_p95.as_micros()
        );
        println!("# dmabuf_fanout 4MB memfd 1x{n} — typed DmaBufPublisher/DmaBufSubscriber");
        Ok(())
    }

    pub(crate) fn run_bench() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<String> = std::env::args().collect();
        let n = match crate::bench_latency::parse_flag(&args, "--n") {
            Some(v) => v,
            None => DEFAULT_N,
        };
        let iters = match crate::bench_latency::parse_flag(&args, "--iters") {
            Some(v) => v,
            None => DEFAULT_ITERS,
        };
        run_inner(n, iters)
    }
}

/// Run the fanout benchmark. Parses `--n` and `--iters` from argv.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    return linux::run_bench();

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("dmabuf fanout benchmark requires Linux");
        std::process::exit(1);
    }
}
