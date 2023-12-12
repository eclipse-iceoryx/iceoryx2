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

use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use iceoryx2_bb_testing::assert_that;
use iceoryx2_pal_concurrency_primitives::semaphore::*;

const TIMEOUT: Duration = Duration::from_millis(25);

#[test]
fn semaphore_post_and_try_wait_works() {
    let initial_value = 5;
    let sut = Semaphore::new(initial_value);

    for _ in 0..initial_value {
        assert_that!(sut.try_wait(), eq true);
    }
    assert_that!(!sut.try_wait(), eq true);

    for _ in 0..initial_value {
        sut.post(|_| {});
    }

    for _ in 0..initial_value {
        assert_that!(sut.try_wait(), eq true);
    }
    assert_that!(!sut.try_wait(), eq true);
}

#[test]
fn semaphore_post_and_wait_works() {
    let initial_value = 5;
    let sut = Semaphore::new(initial_value);

    for _ in 0..initial_value {
        assert_that!(sut.wait(|_, _| false), eq true);
    }
    assert_that!(!sut.wait(|_, _| false), eq true);

    for _ in 0..initial_value {
        sut.post(|_| {});
    }

    for _ in 0..initial_value {
        assert_that!(sut.wait(|_, _| false), eq true);
    }
    assert_that!(!sut.wait(|_, _| false), eq true);
}

#[test]
fn semaphore_wait_blocks() {
    let initial_value = 0;
    let sut = Semaphore::new(initial_value);
    let counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        s.spawn(|| {
            sut.wait(|_, _| true);
            counter.fetch_add(1, Ordering::Relaxed);
        });

        std::thread::sleep(TIMEOUT);
        let old_counter = counter.load(Ordering::Relaxed);
        sut.post(|_| {});

        assert_that!(old_counter, eq 0);
    });
}
