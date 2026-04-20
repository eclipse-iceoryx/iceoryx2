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

/// Shared argument parsing utilities for all dmabuf benchmarks.
#[cfg(target_os = "linux")]
pub(crate) mod args {
    /// Parse `--flag VALUE` from `args`, returning `Some(value)` on success.
    pub(crate) fn parse_flag(args: &[String], flag: &str) -> Option<usize> {
        args.windows(2)
            .find(|w| w[0] == flag)
            .and_then(|w| w[1].parse().ok())
    }
}

#[cfg(target_os = "linux")]
mod bench_fanout;
#[cfg(target_os = "linux")]
mod bench_latency;
#[cfg(target_os = "linux")]
mod bench_throughput;

fn main() {
    #[cfg(target_os = "linux")]
    match std::env::args().nth(1).as_deref() {
        Some("latency") => bench_latency::run_latency(),
        Some("throughput") => bench_throughput::run_throughput(),
        Some("fanout") => bench_fanout::run_fanout(),
        _ => eprintln!("Usage: dmabuf-bench <latency|throughput|fanout>"),
    }

    #[cfg(not(target_os = "linux"))]
    eprintln!(
        "dmabuf-bench: DMA-BUF benchmarks are Linux-only (requires memfd_create + SCM_RIGHTS)"
    );
}
