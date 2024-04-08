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

use iceoryx2_bb_lock_free::mpmc::bit_set::BitSet;

struct Config {
    capacity: usize,
    iterations: usize,
}

fn perform_benchmark_reset_all(config: Config) {
    let sut_a_b = BitSet::new(config.capacity);
    let sut_b_a = BitSet::new(config.capacity);
    // worst case bit - iterate over all bitset to see if its set
    let bit = config.capacity - 1;
    let barrier = Barrier::new(3);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            sut_a_b.set(bit);

            for _ in 0..config.iterations {
                let mut has_update = false;
                while !has_update {
                    sut_b_a.reset_all(|_| {
                        has_update = true;
                    });
                }
                sut_a_b.set(bit);
            }
        });

        let t2 = s.spawn(|| {
            barrier.wait();
            for _ in 0..config.iterations {
                let mut has_update = false;
                while !has_update {
                    sut_a_b.reset_all(|_| {
                        has_update = true;
                    });
                }
                sut_b_a.set(bit);
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
        let start = Instant::now();
        barrier.wait();

        t1.join().expect("thread failure");
        t2.join().expect("thread failure");

        let stop = start.elapsed();
        println!(
            "Bitset::reset_all()  ::: Capacity: {}, Iterations: {}, Time: {}, Latency: {} ns",
            config.capacity,
            config.iterations,
            stop.as_secs_f64(),
            stop.as_nanos() / (config.iterations as u128 * 2)
        );
    });
}

fn perform_benchmark_reset_next(config: Config) {
    let sut_a_b = BitSet::new(config.capacity);
    let sut_b_a = BitSet::new(config.capacity);
    let bit = 0;
    let barrier = Barrier::new(3);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            sut_a_b.set(bit);

            for _ in 0..config.iterations {
                while sut_b_a.reset_next().is_none() {}
                sut_a_b.set(bit);
            }
        });

        let t2 = s.spawn(|| {
            barrier.wait();
            for _ in 0..config.iterations {
                while sut_a_b.reset_next().is_none() {}
                sut_b_a.set(bit);
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
        let start = Instant::now();
        barrier.wait();

        t1.join().expect("thread failure");
        t2.join().expect("thread failure");

        let stop = start.elapsed();
        println!(
            "Bitset::reset_next() ::: Capacity: {}, Iterations: {}, Time: {}, Latency: {} ns",
            config.capacity,
            config.iterations,
            stop.as_secs_f64(),
            stop.as_nanos() / (config.iterations as u128 * 2)
        );
    });
}

fn main() {
    perform_benchmark_reset_all(Config {
        capacity: 256,
        iterations: 2000000,
    });

    perform_benchmark_reset_next(Config {
        capacity: 256,
        iterations: 2000000,
    });
}
