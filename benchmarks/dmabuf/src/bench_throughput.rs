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

//! Throughput benchmark: sustained frame rate for 4 MB memfd DMA-BUF frames.
//!
//! Sends `--frames` frames as fast as possible (publisher side) and receives
//! them on the subscriber side in the same process using the typed
//! `DmaBufPublisher` / `DmaBufSubscriber` convenience layer.
//! Reports total wall-clock time and frames-per-second.
//!
//! # Usage
//!
//! ```text
//! iceoryx2-benchmarks-dmabuf throughput [--frames N]
//! ```
//!
//! Default: 100 000 frames.

#[cfg(target_os = "linux")]
mod linux {
    /// Default number of frames to send.
    const DEFAULT_FRAMES: usize = 100_000;
    /// Maximum spin-poll attempts per frame before treating the receive as stalled.
    const MAX_RECV_POLLS: usize = 1_000_000;
    /// Milliseconds to wait for the UDS fd-channel handshake before benchmarking.
    const SETTLE_MS: u64 = 50;

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

    fn run_inner(frames: usize) -> Result<(), Box<dyn core::error::Error>> {
        use crate::bench_latency::PAYLOAD_BYTES;
        use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};
        use std::time::Instant;

        let svc = "bench/dmabuf/throughput";
        let mut pub_ = DmaBufPublisher::<u64>::create(svc)?;
        let mut sub_ = DmaBufSubscriber::<u64>::create(svc)?;

        settle();

        let start = Instant::now();

        for seq in 0..frames as u64 {
            let buf = crate::common::make_memfd(PAYLOAD_BYTES)?;
            pub_.publish(seq, &buf)?;
            recv_one(&mut sub_)?;
        }

        let elapsed = start.elapsed();
        let fps = frames as f64 / elapsed.as_secs_f64();
        println!(
            "throughput_fps={fps:.1} throughput_total_ms={:.0}",
            elapsed.as_secs_f64() * 1000.0
        );
        println!("# dmabuf_throughput 4MB memfd ({frames} frames) — typed DmaBufPublisher");
        Ok(())
    }

    pub(crate) fn run_bench() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<String> = std::env::args().collect();
        let frames = match crate::bench_latency::parse_flag(&args, "--frames") {
            Some(v) => v,
            None => DEFAULT_FRAMES,
        };
        run_inner(frames)
    }
}

/// Run the throughput benchmark. Parses `--frames` from argv.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    return linux::run_bench();

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("dmabuf throughput benchmark requires Linux");
        std::process::exit(1);
    }
}
