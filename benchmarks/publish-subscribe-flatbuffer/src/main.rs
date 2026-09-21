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

use crate::payload_data_generated::benchmark::{PayloadData, PayloadDataArgs};
use clap::Parser;
use iceoryx2::prelude::*;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::thread::ThreadBuilder;

#[path = "payload_data_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod payload_data_generated;

const ITERATIONS: u64 = 10000000;

const SCHEMA: &[u8] = include_bytes!("payload_data.bfbs");

/// Overhead reserved upfront so that the builder never reallocates during the measurement.
const FLATBUFFER_OVERHEAD: usize = 64;

fn write_schema() -> Result<FilePath, Box<dyn core::error::Error>> {
    let path = std::env::temp_dir().join("iox2_benchmark_payload_data.bfbs");
    std::fs::write(&path, SCHEMA)?;
    Ok(path.to_str().expect("temp dir is valid utf-8").try_into()?)
}

fn perform_benchmark<T: Service + 'static>(
    args: &Args,
    schema_path: &FilePath,
) -> Result<(), Box<dyn core::error::Error>> {
    let service_name_a2b = ServiceName::new("a2b")?;
    let service_name_b2a = ServiceName::new("b2a")?;
    let node = NodeBuilder::new().create::<T>()?;

    let service_a2b = node
        .service_builder(&service_name_a2b)
        .publish_subscribe::<Flatbuffer<PayloadData>>()
        .flatbuffer_schema_path(schema_path)
        .max_publishers(1 + args.number_of_additional_publishers)
        .max_subscribers(1 + args.number_of_additional_subscribers)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(true)
        .create()?;

    let service_b2a = node
        .service_builder(&service_name_b2a)
        .publish_subscribe::<Flatbuffer<PayloadData>>()
        .flatbuffer_schema_path(schema_path)
        .max_publishers(1 + args.number_of_additional_publishers)
        .max_subscribers(1 + args.number_of_additional_subscribers)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(true)
        .create()?;

    let mut additional_publishers = Vec::new();
    let mut additional_subscribers = Vec::new();

    for _ in 0..args.number_of_additional_publishers {
        additional_publishers.push(service_a2b.publisher_builder().create()?);
        additional_publishers.push(service_b2a.publisher_builder().create()?);
    }

    for _ in 0..args.number_of_additional_subscribers {
        additional_subscribers.push(service_a2b.subscriber_builder().create()?);
        additional_subscribers.push(service_b2a.subscriber_builder().create()?);
    }

    // NOTE: the payload is reused every iteration and stays cached.
    let payload = vec![0u8; args.payload_size];
    let reserved_memory = args
        .initial_reserved_memory
        .unwrap_or(args.payload_size + FLATBUFFER_OVERHEAD);

    let start_benchmark_barrier_handle = BarrierHandle::new();
    let startup_barrier_handle = BarrierHandle::new();
    let startup_barrier = BarrierBuilder::new(3)
        .create(&startup_barrier_handle)
        .unwrap();
    let start_benchmark_barrier = BarrierBuilder::new(3)
        .create(&start_benchmark_barrier_handle)
        .unwrap();

    let t1 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_1])
        .priority(255)
        .spawn(|| {
            let sender_a2b = service_a2b
                .publisher_builder()
                .initial_reserved_memory(reserved_memory)
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create()
                .unwrap();
            let receiver_b2a = service_b2a.subscriber_builder().create().unwrap();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                let mut sample = sender_a2b.loan_flatbuffer().unwrap();
                let builder = sample.flatbuffer_builder();

                // BEGIN: standard flatbuffer API
                let data = builder.create_vector(&payload);
                let payload_data =
                    PayloadData::create(builder, &PayloadDataArgs { data: Some(data) });
                // END: standard flatbuffer API

                sample.assume_init(payload_data).send().unwrap();

                loop {
                    if let Some(sample) = receiver_b2a.receive().unwrap() {
                        sample.payload_root().unwrap();
                        break;
                    }
                }
            }
        });

    let t2 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_2])
        .priority(255)
        .spawn(|| {
            let sender_b2a = service_b2a
                .publisher_builder()
                .initial_reserved_memory(reserved_memory)
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create()
                .unwrap();
            let receiver_a2b = service_a2b.subscriber_builder().create().unwrap();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                loop {
                    if let Some(sample) = receiver_a2b.receive().unwrap() {
                        sample.payload_root().unwrap();
                        break;
                    }
                }

                let mut sample = sender_b2a.loan_flatbuffer().unwrap();
                let builder = sample.flatbuffer_builder();

                // BEGIN: standard flatbuffer API
                let data = builder.create_vector(&payload);
                let payload_data =
                    PayloadData::create(builder, &PayloadDataArgs { data: Some(data) });
                // END: standard flatbuffer API

                sample.assume_init(payload_data).send().unwrap();
            }
        });

    startup_barrier.wait();
    let start = Time::now().expect("failed to acquire time");
    start_benchmark_barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "{} ::: Iterations: {}, Time: {} s, Latency: {} ns, Sample Size: {}",
        core::any::type_name::<T>(),
        args.iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (args.iterations as u128 * 2),
        args.payload_size
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
    /// The cpu core that shall be used by participant 1
    #[clap(long, default_value_t = 0)]
    cpu_core_participant_1: usize,
    /// The cpu core that shall be used by participant 2
    #[clap(long, default_value_t = 1)]
    cpu_core_participant_2: usize,
    /// The size in bytes of the flatbuffer payload vector that shall be used
    #[clap(short, long, default_value_t = 8192)]
    payload_size: usize,
    /// The size in bytes that the publisher reserves for the flatbuffer builder. Defaults to the
    /// payload size plus the flatbuffer overhead so that the builder never reallocates.
    #[clap(long)]
    initial_reserved_memory: Option<usize>,
    /// The path to the binary flatbuffer schema that describes the payload type. Defaults to the
    /// schema that is embedded into this benchmark.
    #[clap(long)]
    schema_path: Option<String>,
    /// The number of additional publishers per service in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_publishers: usize,
    /// The number of additional subscribers per service in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_subscribers: usize,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let args = Args::parse();

    if args.debug_mode {
        set_log_level(LogLevel::Trace);
    } else {
        set_log_level(LogLevel::Error);
    }

    let schema_path = match &args.schema_path {
        Some(path) => path.as_str().try_into()?,
        None => write_schema()?,
    };

    let mut at_least_one_benchmark_did_run = false;

    if args.bench_ipc || args.bench_all {
        perform_benchmark::<ipc::Service>(&args, &schema_path)?;
        perform_benchmark::<ipc_threadsafe::Service>(&args, &schema_path)?;
        at_least_one_benchmark_did_run = true;
    }

    if args.bench_local || args.bench_all {
        perform_benchmark::<local::Service>(&args, &schema_path)?;
        perform_benchmark::<local_threadsafe::Service>(&args, &schema_path)?;
        at_least_one_benchmark_did_run = true;
    }

    if !at_least_one_benchmark_did_run {
        println!(
            "Please use either '--bench-all' or select a specific benchmark. See `--help` for details."
        );
    }

    Ok(())
}
