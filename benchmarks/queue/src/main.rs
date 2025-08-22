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
use iceoryx2_bb_lock_free::spsc::index_queue::FixedSizeIndexQueue;
use iceoryx2_bb_lock_free::spsc::queue::Queue;
use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::FixedSizeSafelyOverflowingIndexQueue;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::thread::ThreadBuilder;

const ITERATIONS: u64 = 10000000;

trait PushPop: Send + Sync {
    fn push(&self, value: usize);
    fn pop(&self) -> bool;
}

impl<const CAPACITY: usize> PushPop for Queue<usize, CAPACITY> {
    fn push(&self, value: usize) {
        unsafe { self.push(&value) };
    }

    fn pop(&self) -> bool {
        unsafe { self.pop().is_some() }
    }
}

impl<const CAPACITY: usize> PushPop for FixedSizeIndexQueue<CAPACITY> {
    fn push(&self, value: usize) {
        unsafe { self.push(value as u64) };
    }

    fn pop(&self) -> bool {
        unsafe { self.pop().is_some() }
    }
}

impl<const CAPACITY: usize> PushPop for FixedSizeSafelyOverflowingIndexQueue<CAPACITY> {
    fn push(&self, value: usize) {
        unsafe { self.push(value as u64) };
    }

    fn pop(&self) -> bool {
        unsafe { self.pop().is_some() }
    }
}

fn perform_benchmark<Q: PushPop>(
    args: &Args,
    queue_a2b: Q,
    queue_b2a: Q,
) -> Result<(), Box<dyn core::error::Error>> {
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
            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                queue_a2b.push(0);
                while !queue_b2a.pop() {}
            }
        });

    let t2 = ThreadBuilder::new()
        .affinity(&[args.cpu_core_participant_2])
        .priority(255)
        .spawn(|| {
            startup_barrier.wait();
            start_benchmark_barrier.wait();

            for _ in 0..args.iterations {
                while !queue_a2b.pop() {}

                queue_b2a.push(0);
            }
        });

    startup_barrier.wait();
    let start = Time::now().expect("failed to acquire time");
    start_benchmark_barrier.wait();

    drop(t1);
    drop(t2);

    let stop = start.elapsed().expect("failed to measure time");
    println!(
        "{} ::: Iterations: {}, Time: {} s, Latency: {} ns",
        core::any::type_name::<Q>(),
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
    /// The cpu core that shall be used by participant 1
    #[clap(long, default_value_t = 0)]
    cpu_core_participant_1: usize,
    /// The cpu core that shall be used by participant 2
    #[clap(long, default_value_t = 1)]
    cpu_core_participant_2: usize,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let args = Args::parse();

    perform_benchmark(&args, Queue::<usize, 1>::new(), Queue::<usize, 1>::new())?;
    perform_benchmark(&args, Queue::<usize, 2>::new(), Queue::<usize, 2>::new())?;
    perform_benchmark(&args, Queue::<usize, 16>::new(), Queue::<usize, 16>::new())?;
    perform_benchmark(
        &args,
        Queue::<usize, 128>::new(),
        Queue::<usize, 128>::new(),
    )?;

    perform_benchmark(
        &args,
        FixedSizeIndexQueue::<1>::new(),
        FixedSizeIndexQueue::<1>::new(),
    )?;
    perform_benchmark(
        &args,
        FixedSizeIndexQueue::<2>::new(),
        FixedSizeIndexQueue::<2>::new(),
    )?;
    perform_benchmark(
        &args,
        FixedSizeIndexQueue::<16>::new(),
        FixedSizeIndexQueue::<16>::new(),
    )?;
    perform_benchmark(
        &args,
        FixedSizeIndexQueue::<128>::new(),
        FixedSizeIndexQueue::<128>::new(),
    )?;

    perform_benchmark(
        &args,
        FixedSizeSafelyOverflowingIndexQueue::<1>::new(),
        FixedSizeSafelyOverflowingIndexQueue::<1>::new(),
    )?;
    perform_benchmark(
        &args,
        FixedSizeSafelyOverflowingIndexQueue::<2>::new(),
        FixedSizeSafelyOverflowingIndexQueue::<2>::new(),
    )?;
    perform_benchmark(
        &args,
        FixedSizeSafelyOverflowingIndexQueue::<16>::new(),
        FixedSizeSafelyOverflowingIndexQueue::<16>::new(),
    )?;
    perform_benchmark(
        &args,
        FixedSizeSafelyOverflowingIndexQueue::<128>::new(),
        FixedSizeSafelyOverflowingIndexQueue::<128>::new(),
    )?;

    Ok(())
}
