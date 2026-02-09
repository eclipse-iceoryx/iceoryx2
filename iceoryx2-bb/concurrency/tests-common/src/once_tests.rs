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
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_nostd_macros::requires_std;

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

#[requires_std("threading")]
pub fn once_works_with_multiple_threads() {
    use iceoryx2_bb_concurrency::internal::strategy::barrier::Barrier;

    const NUMBER_OF_THREADS: u32 = 10;

    let once = Once::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS + 1);
    let counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                once.call_once(|| {
                    counter.fetch_add(1, Ordering::Relaxed);
                });
            });
        }

        barrier.wait(|_, _| {}, |_| {});
    });

    assert_that!(counter.load(Ordering::Relaxed), eq 1);
    assert_that!(once.is_completed(), eq true);
}

pub fn once_is_completed_returns_false_initially() {
    let once = Once::new();
    assert_that!(once.is_completed(), eq false);
}
