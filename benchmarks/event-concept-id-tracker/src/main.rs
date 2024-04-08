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
use iceoryx2_bb_memory::{bump_allocator::BumpAllocator, memory::Memory};
use pin_init::PtrPinWith;
use std::{sync::Barrier, time::Instant};

use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
use iceoryx2_cal::event::id_tracker::IdTracker;

fn perform_benchmark_acquire_all<T: IdTracker>(args: &Args) {
    let memory = Box::pin_with(Memory::<10485760, BumpAllocator>::new()).unwrap();
    let sut_a_b = unsafe { T::new_uninit(args.capacity) };
    let sut_b_a = unsafe { T::new_uninit(args.capacity) };

    unsafe { sut_a_b.init(memory.allocator()).expect("sut init failed") };
    unsafe { sut_b_a.init(memory.allocator()).expect("sut init failed") };

    // worst case bit - iterate over all bitset to see if its set
    let bit = sut_a_b.trigger_id_max();
    let barrier = Barrier::new(3);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            let _ = unsafe { sut_a_b.add(bit) };

            for _ in 0..args.iterations {
                let mut has_update = false;
                while !has_update {
                    unsafe {
                        sut_b_a.acquire_all(|_| {
                            has_update = true;
                        })
                    };
                }
                let _ = unsafe { sut_a_b.add(bit) };
            }
        });

        let t2 = s.spawn(|| {
            barrier.wait();
            for _ in 0..args.iterations {
                let mut has_update = false;
                while !has_update {
                    unsafe {
                        sut_a_b.acquire_all(|_| {
                            has_update = true;
                        })
                    };
                }
                let _ = unsafe { sut_b_a.add(bit) };
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
        let start = Instant::now();
        barrier.wait();

        t1.join().expect("thread failure");
        t2.join().expect("thread failure");

        let stop = start.elapsed();
        println!(
            "{}::acquire_all()  ::: MaxTriggerId: {:?}, Iterations: {}, Time: {}, Latency: {} ns",
            std::any::type_name::<T>(),
            sut_a_b.trigger_id_max(),
            args.iterations,
            stop.as_secs_f64(),
            stop.as_nanos() / (args.iterations as u128 * 2)
        );
    });
}

fn perform_benchmark_acquire_next<T: IdTracker>(args: &Args) {
    let memory = Box::pin_with(Memory::<10485760, BumpAllocator>::new()).unwrap();
    let sut_a_b = unsafe { T::new_uninit(args.capacity) };
    let sut_b_a = unsafe { T::new_uninit(args.capacity) };

    unsafe { sut_a_b.init(memory.allocator()).expect("sut init failed") };
    unsafe { sut_b_a.init(memory.allocator()).expect("sut init failed") };

    // worst case bit - iterate over all bitset to see if its set
    let bit = sut_a_b.trigger_id_max();
    let barrier = Barrier::new(3);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            let _ = unsafe { sut_a_b.add(bit) };

            for _ in 0..args.iterations {
                while unsafe { sut_b_a.acquire().is_none() } {}
                let _ = unsafe { sut_a_b.add(bit) };
            }
        });

        let t2 = s.spawn(|| {
            barrier.wait();
            for _ in 0..args.iterations {
                while unsafe { sut_a_b.acquire().is_none() } {}
                let _ = unsafe { sut_b_a.add(bit) };
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
        let start = Instant::now();
        barrier.wait();

        t1.join().expect("thread failure");
        t2.join().expect("thread failure");

        let stop = start.elapsed();
        println!(
            "{}::acquire_next()  ::: MaxTriggerId: {:?}, Iterations: {}, Time: {}, Latency: {} ns",
            std::any::type_name::<T>(),
            sut_a_b.trigger_id_max(),
            args.iterations,
            stop.as_secs_f64(),
            stop.as_nanos() / (args.iterations as u128 * 2)
        );
    });
}

const ITERATIONS: usize = 1000000;
const CAPACITY: usize = 128;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Number of iterations the A --> B --> A communication is repeated
    #[clap(short, long, default_value_t = ITERATIONS)]
    iterations: usize,
    /// Capacity of the bitset
    #[clap(short, long, default_value_t = CAPACITY)]
    capacity: usize,
}

fn main() {
    let args = Args::parse();

    perform_benchmark_acquire_all::<RelocatableBitSet>(&args);
    perform_benchmark_acquire_next::<RelocatableBitSet>(&args);
}
