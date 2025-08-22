// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

fn perform_benchmark<T: Service>(args: &Args) -> Result<(), Box<dyn core::error::Error>> {
    let service_name_a2b = ServiceName::new("a2b")?;
    let service_name_b2a = ServiceName::new("b2a")?;
    let node = NodeBuilder::new().create::<T>()?;

    let service_a2b = node
        .service_builder(&service_name_a2b)
        .event()
        .max_notifiers(1 + args.number_of_additional_notifiers)
        .max_listeners(1 + args.number_of_additional_listeners)
        .event_id_max_value(args.max_event_id)
        .create()
        .unwrap();

    let service_b2a = node
        .service_builder(&service_name_b2a)
        .event()
        .max_notifiers(1 + args.number_of_additional_notifiers)
        .max_listeners(1 + args.number_of_additional_listeners)
        .event_id_max_value(args.max_event_id)
        .create()
        .unwrap();

    let mut additional_notifiers = Vec::new();
    let mut additional_listeners = Vec::new();

    for _ in 0..args.number_of_additional_notifiers {
        additional_notifiers.push(service_a2b.notifier_builder().create()?);
        additional_notifiers.push(service_b2a.notifier_builder().create()?);
    }

    for _ in 0..args.number_of_additional_listeners {
        additional_listeners.push(service_a2b.listener_builder().create()?);
        additional_listeners.push(service_b2a.listener_builder().create()?);
    }

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
            let notifier_a2b = service_a2b.notifier_builder().create().unwrap();
            let listener_b2a = service_b2a.listener_builder().create().unwrap();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            notifier_a2b.notify().expect("failed to notify");

            for _ in 0..args.iterations {
                while listener_b2a.blocking_wait_one().unwrap().is_none() {}
                notifier_a2b.notify().expect("failed to notify");
            }
        });

    let t2 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_2])
        .priority(255)
        .spawn(|| {
            let notifier_b2a = service_b2a.notifier_builder().create().unwrap();
            let listener_a2b = service_a2b.listener_builder().create().unwrap();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                while listener_a2b.blocking_wait_one().unwrap().is_none() {}
                notifier_b2a.notify().expect("failed to notify");
            }
        });

    startup_barrier.wait();
    let start = Time::now().expect("failed to acquire time");
    start_benchmark_barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "{} ::: MaxEventId: {}, Iterations: {}, Time: {} s, Latency: {} ns",
        core::any::type_name::<T>(),
        args.max_event_id,
        args.iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (args.iterations as u128 * 2)
    );

    Ok(())
}

const ITERATIONS: usize = 1000000;
const EVENT_ID_MAX_VALUE: usize = 128;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: usize,
    /// Run benchmark for every service setup
    #[clap(short, long)]
    bench_all: bool,
    /// Run benchmark for the IPC zero copy setup
    #[clap(long)]
    bench_ipc: bool,
    /// Run benchmark for the process local setup
    #[clap(long)]
    bench_local: bool,
    /// The greatest supported EventId
    #[clap(short, long, default_value_t = EVENT_ID_MAX_VALUE)]
    max_event_id: usize,
    /// Activate full log output
    #[clap(short, long)]
    debug_mode: bool,
    /// The cpu core that shall be used by participant 1
    #[clap(long, default_value_t = 0)]
    cpu_core_participant_1: usize,
    /// The cpu core that shall be used by participant 2
    #[clap(long, default_value_t = 1)]
    cpu_core_participant_2: usize,
    /// The number of additional notifiers per service in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_notifiers: usize,
    /// The number of additional listeners per service in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_listeners: usize,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let args = Args::parse();

    if args.debug_mode {
        set_log_level(iceoryx2_bb_log::LogLevel::Trace);
    } else {
        set_log_level(iceoryx2_bb_log::LogLevel::Error);
    }

    let mut at_least_one_benchmark_did_run = false;

    if args.bench_ipc || args.bench_all {
        perform_benchmark::<ipc::Service>(&args)?;
        perform_benchmark::<ipc_threadsafe::Service>(&args)?;
        at_least_one_benchmark_did_run = true;
    }

    if args.bench_local || args.bench_all {
        perform_benchmark::<local::Service>(&args)?;
        perform_benchmark::<local_threadsafe::Service>(&args)?;
        at_least_one_benchmark_did_run = true;
    }

    if !at_least_one_benchmark_did_run {
        println!(
            "Please use either '--bench-all' or select a specific benchmark. See `--help` for details."
        );
    }

    Ok(())
}
