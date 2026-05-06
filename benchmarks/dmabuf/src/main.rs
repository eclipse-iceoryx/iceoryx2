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

//! CLI dispatcher: `cargo run -p iceoryx2-benchmarks-dmabuf --release -- {latency|throughput|fanout}`

mod bench_fanout;
mod bench_latency;
mod bench_throughput;
#[cfg(target_os = "linux")]
mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("latency") => bench_latency::run(),
        Some("throughput") => bench_throughput::run(),
        Some("fanout") => bench_fanout::run(),
        Some(other) => {
            eprintln!("unknown bench '{other}'; expected: latency | throughput | fanout");
            std::process::exit(2);
        }
        None => {
            eprintln!("usage: iceoryx2-benchmarks-dmabuf {{latency|throughput|fanout}}");
            std::process::exit(2);
        }
    }
}
