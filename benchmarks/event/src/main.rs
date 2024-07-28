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

fn perform_benchmark<T: Service>(args: &Args) {
    let service_name_a2b = ServiceName::new("a2b").unwrap();
    let service_name_b2a = ServiceName::new("b2a").unwrap();
    let node = NodeBuilder::new().create::<T>().unwrap();

    let service_a2b = node
        .service_builder(&service_name_a2b)
        .event()
        .max_notifiers(1)
        .max_listeners(1)
        .event_id_max_value(args.max_event_id)
        .create()
        .unwrap();

    let service_b2a = node
        .service_builder(&service_name_b2a)
        .event()
        .max_notifiers(1)
        .max_listeners(1)
        .event_id_max_value(args.max_event_id)
        .create()
        .unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(3).create(&barrier_handle).unwrap();

    let t1 = ThreadBuilder::new().affinity(0).priority(255).spawn(|| {
        let notifier_a2b = service_a2b.notifier_builder().create().unwrap();
        let listener_b2a = service_b2a.listener_builder().create().unwrap();

        barrier.wait();
        notifier_a2b.notify().expect("failed to notify");

        for _ in 0..args.iterations {
            while listener_b2a.blocking_wait_one().unwrap().is_none() {}
            notifier_a2b.notify().expect("failed to notify");
        }
    });

    let t2 = ThreadBuilder::new().affinity(1).priority(255).spawn(|| {
        let notifier_b2a = service_b2a.notifier_builder().create().unwrap();
        let listener_a2b = service_a2b.listener_builder().create().unwrap();

        barrier.wait();
        for _ in 0..args.iterations {
            while listener_a2b.blocking_wait_one().unwrap().is_none() {}
            notifier_b2a.notify().expect("failed to notify");
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(100));
    let start = Time::now().expect("failed to acquire time");
    barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "{} ::: MaxEventId: {}, Iterations: {}, Time: {}, Latency: {} ns",
        std::any::type_name::<T>(),
        args.max_event_id,
        args.iterations,
        stop.as_secs_f64(),
        stop.as_nanos() / (args.iterations as u128 * 2)
    );
}

const ITERATIONS: usize = 1000000;
const EVENT_ID_MAX_VALUE: usize = 128;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: usize,
    /// The greatest supported EventId
    #[clap(short, long, default_value_t = EVENT_ID_MAX_VALUE)]
    max_event_id: usize,
    /// Activate full log output
    #[clap(short, long)]
    debug_mode: bool,
}

fn main() {
    let args = Args::parse();

    if args.debug_mode {
        set_log_level(iceoryx2_bb_log::LogLevel::Trace);
    } else {
        set_log_level(iceoryx2_bb_log::LogLevel::Info);
    }

    perform_benchmark::<ipc::Service>(&args);
    perform_benchmark::<local::Service>(&args);
}
