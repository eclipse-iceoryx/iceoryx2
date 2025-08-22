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

fn perform_response_stream_benchmark<T: Service>(
    args: &Args,
) -> Result<(), Box<dyn core::error::Error>> {
    let service_name_a2b = ServiceName::new("a2b")?;
    let service_name_b2a = ServiceName::new("b2a")?;
    let node = NodeBuilder::new().create::<T>()?;

    let service_a2b = node
        .service_builder(&service_name_a2b)
        .request_response::<u64, u64>()
        .max_servers(1 + args.number_of_additional_servers)
        .max_clients(1 + args.number_of_additional_clients)
        .max_response_buffer_size(1)
        .create()?;

    let service_b2a = node
        .service_builder(&service_name_b2a)
        .request_response::<u64, u64>()
        .max_servers(1 + args.number_of_additional_servers)
        .max_clients(1 + args.number_of_additional_clients)
        .max_response_buffer_size(1)
        .create()?;

    let mut additional_clients = Vec::new();
    let mut additional_servers = Vec::new();

    for _ in 0..args.number_of_additional_clients {
        additional_clients.push(service_a2b.client_builder().create()?);
        additional_clients.push(service_b2a.client_builder().create()?);
    }

    for _ in 0..args.number_of_additional_servers {
        additional_servers.push(service_a2b.server_builder().create()?);
        additional_servers.push(service_b2a.server_builder().create()?);
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
            let client_a2b = service_a2b.client_builder().create().unwrap();
            let server_b2a = service_b2a.server_builder().create().unwrap();

            startup_barrier.wait();
            let pending_response = client_a2b.send_copy(0).unwrap();
            let active_request = {
                while !server_b2a.has_requests().unwrap() {}
                server_b2a.receive().unwrap().unwrap()
            };
            start_benchmark_barrier.wait();

            let mut response = unsafe { active_request.loan_uninit().unwrap().assume_init() };

            for _ in 0..args.iterations {
                response.send().unwrap();
                response = unsafe { active_request.loan_uninit().unwrap().assume_init() };
                while pending_response.receive().unwrap().is_none() {}
            }
        });

    let t2 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_2])
        .priority(255)
        .spawn(|| {
            let server_a2b = service_a2b.server_builder().create().unwrap();
            let client_b2a = service_b2a.client_builder().create().unwrap();

            startup_barrier.wait();
            let pending_response = client_b2a.send_copy(0).unwrap();
            let active_request = {
                while !server_a2b.has_requests().unwrap() {}
                server_a2b.receive().unwrap().unwrap()
            };
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                let response = unsafe { active_request.loan_uninit().unwrap().assume_init() };
                while pending_response.receive().unwrap().is_none() {}

                response.send().unwrap();
            }
        });

    startup_barrier.wait();
    let start = Time::now().expect("failed to acquire time");
    start_benchmark_barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "[RESPONSE_STREAM] {} ::: Iterations: {}, Time: {} s, Latency: {} ns",
        core::any::type_name::<T>(),
        args.iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (args.iterations as u128 * 2),
    );

    Ok(())
}

fn perform_request_benchmark<T: Service>(args: &Args) -> Result<(), Box<dyn core::error::Error>> {
    let service_name_a2b = ServiceName::new("a2b")?;
    let service_name_b2a = ServiceName::new("b2a")?;
    let node = NodeBuilder::new().create::<T>()?;

    let service_a2b = node
        .service_builder(&service_name_a2b)
        .request_response::<u64, u64>()
        .max_servers(1 + args.number_of_additional_servers)
        .max_clients(1 + args.number_of_additional_clients)
        .max_response_buffer_size(1)
        .create()?;

    let service_b2a = node
        .service_builder(&service_name_b2a)
        .request_response::<u64, u64>()
        .max_servers(1 + args.number_of_additional_servers)
        .max_clients(1 + args.number_of_additional_clients)
        .max_response_buffer_size(1)
        .create()?;

    let mut additional_clients = Vec::new();
    let mut additional_servers = Vec::new();

    for _ in 0..args.number_of_additional_clients {
        additional_clients.push(service_a2b.client_builder().create()?);
        additional_clients.push(service_b2a.client_builder().create()?);
    }

    for _ in 0..args.number_of_additional_servers {
        additional_servers.push(service_a2b.server_builder().create()?);
        additional_servers.push(service_b2a.server_builder().create()?);
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
            let client_a2b = service_a2b.client_builder().create().unwrap();
            let server_b2a = service_b2a.server_builder().create().unwrap();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            let mut request = unsafe { client_a2b.loan_uninit().unwrap().assume_init() };

            for _ in 0..args.iterations {
                request.send().unwrap();
                request = unsafe { client_a2b.loan_uninit().unwrap().assume_init() };
                while server_b2a.receive().unwrap().is_none() {}
            }
        });

    let t2 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_2])
        .priority(255)
        .spawn(|| {
            let client_b2a = service_b2a.client_builder().create().unwrap();
            let server_a2b = service_a2b.server_builder().create().unwrap();

            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                let request = unsafe { client_b2a.loan_uninit().unwrap().assume_init() };
                while server_a2b.receive().unwrap().is_none() {}

                request.send().unwrap();
            }
        });

    startup_barrier.wait();
    let start = Time::now().expect("failed to acquire time");
    start_benchmark_barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "[REQUESTS] {} ::: Iterations: {}, Time: {} s, Latency: {} ns",
        core::any::type_name::<T>(),
        args.iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (args.iterations as u128 * 2),
    );

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: u64,
    /// Activate full log output
    #[clap(short, long)]
    debug_mode: bool,
    /// The cpu core that shall be used by participant 1
    #[clap(long, default_value_t = 0)]
    cpu_core_participant_1: usize,
    /// The cpu core that shall be used by participant 2
    #[clap(long, default_value_t = 1)]
    cpu_core_participant_2: usize,
    /// The number of additional servers per service in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_servers: usize,
    /// The number of additional clients per service in the setup.
    #[clap(long, default_value_t = 0)]
    number_of_additional_clients: usize,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let args = Args::parse();

    if args.debug_mode {
        set_log_level(iceoryx2_bb_log::LogLevel::Trace);
    } else {
        set_log_level(iceoryx2_bb_log::LogLevel::Error);
    }

    perform_request_benchmark::<ipc::Service>(&args)?;
    perform_request_benchmark::<ipc_threadsafe::Service>(&args)?;
    perform_request_benchmark::<local::Service>(&args)?;
    perform_request_benchmark::<local_threadsafe::Service>(&args)?;
    perform_response_stream_benchmark::<ipc::Service>(&args)?;
    perform_response_stream_benchmark::<ipc_threadsafe::Service>(&args)?;
    perform_response_stream_benchmark::<local::Service>(&args)?;
    perform_response_stream_benchmark::<local_threadsafe::Service>(&args)?;

    Ok(())
}
