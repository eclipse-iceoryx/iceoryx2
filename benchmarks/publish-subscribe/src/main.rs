// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use clap::Parser;
use iceoryx2::prelude::*;
use iceoryx2_bb_log::set_log_level;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::thread::ThreadBuilder;

const ITERATIONS: u64 = 10000000;

fn perform_benchmark<T: Service>(iterations: u64) -> Result<(), Box<dyn std::error::Error>> {
    let service_name_a2b = ServiceName::new("a2b")?;
    let service_name_b2a = ServiceName::new("b2a")?;
    let node = NodeBuilder::new().create::<T>()?;

    let service_a2b = node
        .service_builder(&service_name_a2b)
        .publish_subscribe::<u64>()
        .max_publishers(1)
        .max_subscribers(1)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(true)
        .create()?;

    let service_b2a = node
        .service_builder(&service_name_b2a)
        .publish_subscribe::<u64>()
        .max_publishers(1)
        .max_subscribers(1)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(true)
        .create()?;

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(3).create(&barrier_handle).unwrap();

    let t1 = ThreadBuilder::new().affinity(0).priority(255).spawn(|| {
        let sender_a2b = service_a2b.publisher_builder().create().unwrap();
        let receiver_b2a = service_b2a.subscriber_builder().create().unwrap();

        barrier.wait();

        let mut sample = sender_a2b.loan().unwrap();

        for _ in 0..iterations {
            sample.send().unwrap();
            sample = sender_a2b.loan().unwrap();
            while receiver_b2a.receive().unwrap().is_none() {}
        }
    });

    let t2 = ThreadBuilder::new().affinity(1).priority(255).spawn(|| {
        let sender_b2a = service_b2a.publisher_builder().create().unwrap();
        let receiver_a2b = service_a2b.subscriber_builder().create().unwrap();

        barrier.wait();

        for _ in 0..iterations {
            let sample = sender_b2a.loan().unwrap();
            while receiver_a2b.receive().unwrap().is_none() {}

            sample.send().unwrap();
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(100));
    let start = Time::now().expect("failed to acquire time");
    barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "{} ::: Iterations: {}, Time: {}, Latency: {} ns",
        std::any::type_name::<T>(),
        iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (iterations as u128 * 2)
    );

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: u64,
    /// Run benchmark for every service setup
    #[clap(short, long)]
    bench_all: bool,
    /// Run benchmark for the IPC zero copy setup
    #[clap(long)]
    bench_ipc: bool,
    /// Run benchmark for the process local setup
    #[clap(long)]
    bench_local: bool,
    /// Activate full log output
    #[clap(short, long)]
    debug_mode: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.debug_mode {
        set_log_level(iceoryx2_bb_log::LogLevel::Trace);
    } else {
        set_log_level(iceoryx2_bb_log::LogLevel::Info);
    }

    let mut at_least_one_benchmark_did_run = false;

    if args.bench_ipc || args.bench_all {
        perform_benchmark::<ipc::Service>(args.iterations)?;
        at_least_one_benchmark_did_run = true;
    }

    if args.bench_local || args.bench_all {
        perform_benchmark::<local::Service>(args.iterations)?;
        at_least_one_benchmark_did_run = true;
    }

    if !at_least_one_benchmark_did_run {
        println!(
            "Please use either '--bench_all' or select a specific benchmark. See `--help` for details."
        );
    }

    Ok(())
}
