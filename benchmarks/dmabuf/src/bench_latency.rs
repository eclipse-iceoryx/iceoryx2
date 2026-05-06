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

//! Latency benchmark: round-trip time for a single 4 MB memfd DMA-BUF frame.
//!
//! Measures publisher→subscriber end-to-end latency (send + recv) over
//! `--iters` iterations after `--warmup` warm-up rounds using the typed
//! `DmaBufPublisher` / `DmaBufSubscriber` convenience layer.  Reports
//! p50, p95, p99, and max in microseconds.
//!
//! # Usage
//!
//! ```text
//! iceoryx2-benchmarks-dmabuf latency [--iters N] [--warmup N]
//! ```
//!
//! Default: 10 000 iterations, 100 warm-up iterations.

#[cfg(target_os = "linux")]
pub(crate) use linux::PAYLOAD_BYTES;

#[cfg(target_os = "linux")]
pub(crate) use linux::parse_flag;

#[cfg(target_os = "linux")]
mod linux {
    /// Frame payload size in bytes (4 MB memfd).
    pub(crate) const PAYLOAD_BYTES: i64 = 4 * 1024 * 1024;
    /// Default number of warm-up rounds before measurement starts.
    const DEFAULT_WARMUP: usize = 100;
    /// Default number of measured iterations.
    const DEFAULT_ITERS: usize = 10_000;
    /// Maximum spin-poll attempts per frame before treating the receive as stalled.
    const MAX_RECV_POLLS: usize = 1_000_000;
    /// Milliseconds to wait for the UDS fd-channel handshake before benchmarking.
    const SETTLE_MS: u64 = 50;

    /// Parse `--flag VALUE` from the process argument list, returning the value if
    /// found and successfully parsed as `usize`.
    pub(crate) fn parse_flag(args: &[String], flag: &str) -> Option<usize> {
        args.windows(2)
            .find(|w| w[0] == flag)
            .and_then(|w| w[1].parse().ok())
    }

    /// Blocking settle: waits for the UDS fd-channel handshake to complete.
    ///
    /// This benchmark is a synchronous single-threaded binary with no async
    /// runtime; blocking is correct and intentional here.
    fn settle() {
        // BENCHMARK: synchronous settle delay — not async, blocking is correct.
        let dur = std::time::Duration::from_millis(SETTLE_MS);
        std::thread::sleep(dur);
    }

    fn recv_one(
        sub_: &mut iceoryx2_dmabuf::DmaBufSubscriber<u64>,
    ) -> Result<(), Box<dyn core::error::Error>> {
        for _ in 0..MAX_RECV_POLLS {
            if sub_.receive()?.is_some() {
                return Ok(());
            }
        }
        Err("receive stalled: no sample after max polls".into())
    }

    pub(crate) fn run_inner(
        iters: usize,
        warmup: usize,
    ) -> Result<(), Box<dyn core::error::Error>> {
        use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};
        use std::time::Instant;

        let svc = "bench/dmabuf/latency";
        let mut pub_ = DmaBufPublisher::<u64>::create(svc)?;
        let mut sub_ = DmaBufSubscriber::<u64>::create(svc)?;

        settle();

        let total = warmup + iters;
        let mut samples = Vec::with_capacity(iters);

        for i in 0..total {
            let buf = crate::common::make_memfd(PAYLOAD_BYTES)?;

            let t0 = Instant::now();
            pub_.publish(i as u64, &buf)?;
            recv_one(&mut sub_)?;
            let elapsed = t0.elapsed();

            if i >= warmup {
                samples.push(elapsed);
            }
        }

        samples.sort_unstable();
        let p50 = samples[iters / 2];
        let p95 = samples[(iters * 95) / 100];
        let p99 = samples[(iters * 99) / 100];
        let max = *samples.last().ok_or("no samples collected")?;

        println!(
            "latency_p50_us={} latency_p95_us={} latency_p99_us={} latency_max_us={}",
            p50.as_micros(),
            p95.as_micros(),
            p99.as_micros(),
            max.as_micros(),
        );
        println!(
            "# dmabuf_latency 4MB memfd ({iters} iters, {warmup} warmup) — typed DmaBufPublisher"
        );
        Ok(())
    }

    pub(crate) fn run_bench() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<String> = std::env::args().collect();
        let iters = match parse_flag(&args, "--iters") {
            Some(v) => v,
            None => DEFAULT_ITERS,
        };
        let warmup = match parse_flag(&args, "--warmup") {
            Some(v) => v,
            None => DEFAULT_WARMUP,
        };
        run_inner(iters, warmup)
    }
}

/// Run the latency benchmark. Parses `--iters` and `--warmup` from argv.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    return linux::run_bench();

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("dmabuf latency benchmark requires Linux");
        std::process::exit(1);
    }
}
