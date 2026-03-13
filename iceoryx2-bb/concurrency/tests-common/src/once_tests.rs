// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_concurrency::atomic::{AtomicU32, Ordering};

use iceoryx2_bb_concurrency::once::Once;
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;

pub fn once_executes_exactly_once() {
    let once = Once::new();
    let counter = AtomicU32::new(0);

    once.call_once(|| {
        counter.fetch_add(1, Ordering::Relaxed);
    });

    once.call_once(|| {
        counter.fetch_add(1, Ordering::Relaxed);
    });

    once.call_once(|| {
        counter.fetch_add(1, Ordering::Relaxed);
    });

    assert_that!(counter.load(Ordering::Relaxed), eq 1);
    assert_that!(once.is_completed(), eq true);
}

pub fn once_works_with_multiple_threads() {
    const NUMBER_OF_THREADS: u32 = 10;

    let once = Once::new();
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(NUMBER_OF_THREADS + 1)
        .create(&barrier_handle)
        .unwrap();
    let counter = AtomicU32::new(0);

    thread_scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    once.call_once(|| {
                        counter.fetch_add(1, Ordering::Relaxed);
                    });
                })
                .expect("failed to spawn thread");
        }

        barrier.wait();

        Ok(())
    })
    .expect("failed to spawn thread");

    assert_that!(counter.load(Ordering::Relaxed), eq 1);
    assert_that!(once.is_completed(), eq true);
}

pub fn once_is_completed_returns_false_initially() {
    let once = Once::new();
    assert_that!(once.is_completed(), eq false);
}
