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

#![allow(clippy::disallowed_types)]

use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn blocks() -> Result<(), BarrierCreationError> {
    let _watchdog = Watchdog::new();

    let handle = BarrierHandle::new();
    let handle2 = BarrierHandle::new();
    let handle3 = BarrierHandle::new();
    let sut = BarrierBuilder::new(3).create(&handle)?;
    let sut2 = BarrierBuilder::new(3).create(&handle2)?;
    let sut3 = BarrierBuilder::new(3).create(&handle3)?;
    let counter = AtomicU64::new(0);

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                sut.wait();
                sut2.wait();
                counter.fetch_add(10, Ordering::Relaxed);
                sut3.wait();
            })
            .expect("failed to spawn thread");
        s.thread_builder()
            .spawn(|| {
                sut.wait();
                sut2.wait();
                counter.fetch_add(10, Ordering::Relaxed);
                sut3.wait();
            })
            .expect("failed to spawn thread");

        sut.wait();
        let counter_old = counter.load(Ordering::Relaxed);
        sut2.wait();
        sut3.wait();

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 20);

        Ok(())
    })
    .expect("failed to spawn thread");

    Ok(())
}

#[test]
pub fn resets_when_the_one_and_only_waiter_has_woken_up() -> Result<(), BarrierCreationError> {
    let _watchdog = Watchdog::new();
    const ITERATIONS: u64 = 10;

    let handle = BarrierHandle::new();
    let sut = BarrierBuilder::new(1).create(&handle)?;
    let mut counter = 0;

    for _ in 0..ITERATIONS {
        sut.wait();
        counter += 1;
    }

    assert_that!(counter, eq ITERATIONS);

    Ok(())
}

#[test]
pub fn resets_when_all_waiters_have_woken_up() -> Result<(), BarrierCreationError> {
    let _watchdog = Watchdog::new();
    const ITERATIONS: u64 = 10;
    const NUMBER_OF_THREADS: usize = 8;

    for n in 1..NUMBER_OF_THREADS {
        let handle = BarrierHandle::new();
        let sut = BarrierBuilder::new(n as _).create(&handle)?;
        let counter = AtomicU64::new(0);

        thread_scope(|s| {
            for _ in 0..n {
                s.thread_builder().spawn(|| {
                    for _ in 0..ITERATIONS {
                        sut.wait();
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                })?;
            }
            Ok(())
        })
        .unwrap();
        assert_that!(counter.load(Ordering::Relaxed), eq n as u64 * ITERATIONS);
    }

    Ok(())
}
