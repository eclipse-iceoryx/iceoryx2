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

use std::{sync::Barrier, time::Instant};

use clap::Parser;
use iceoryx2::prelude::*;
use iceoryx2_bb_log::set_log_level;

fn perform_benchmark<T: Service>(args: &Args) {
    let service_name_a2b = ServiceName::new("a2b").unwrap();
    let service_name_b2a = ServiceName::new("b2a").unwrap();

    let service_a2b = T::new(&service_name_a2b)
        .event()
        .max_notifiers(1)
        .max_listeners(1)
        .max_event_id(args.max_event_id)
        .create()
        .unwrap();

    let service_b2a = T::new(&service_name_b2a)
        .event()
        .max_notifiers(1)
        .max_listeners(1)
        .max_event_id(args.max_event_id)
        .create()
        .unwrap();

    let barrier = Barrier::new(3);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            let notifier_a2b = service_a2b.notifier().create().unwrap();
            let mut listener_b2a = service_b2a.listener().create().unwrap();

            barrier.wait();
            notifier_a2b.notify().expect("failed to notify");

            for _ in 0..args.iterations {
                while listener_b2a.blocking_wait().unwrap().is_none() {}
                notifier_a2b.notify().expect("failed to notify");
            }
        });

        let t2 = s.spawn(|| {
            let notifier_b2a = service_b2a.notifier().create().unwrap();
            let mut listener_a2b = service_a2b.listener().create().unwrap();

            barrier.wait();
            for _ in 0..args.iterations {
                while listener_a2b.blocking_wait().unwrap().is_none() {}
                notifier_b2a.notify().expect("failed to notify");
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
        let start = Instant::now();
        barrier.wait();

        t1.join().expect("thread failure");
        t2.join().expect("thread failure");

        let stop = start.elapsed();
        println!(
            "{} ::: MaxEventId: {}, Iterations: {}, Time: {}, Latency: {} ns",
            std::any::type_name::<T>(),
            args.max_event_id,
            args.iterations,
            stop.as_secs_f64(),
            stop.as_nanos() / (args.iterations as u128 * 2)
        );
    });
}

const ITERATIONS: usize = 1000000;
const MAX_EVENT_ID: u64 = 128;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: usize,
    /// The greatest supported EventId
    #[clap(short, long, default_value_t = MAX_EVENT_ID)]
    max_event_id: u64,
}

fn main() {
    let args = Args::parse();
    set_log_level(iceoryx2_bb_log::LogLevel::Error);

    perform_benchmark::<zero_copy::Service>(&args);
    perform_benchmark::<process_local::Service>(&args);
}
