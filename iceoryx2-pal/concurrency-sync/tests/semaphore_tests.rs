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

use core::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use iceoryx2_pal_concurrency_sync::{semaphore::*, WaitAction, WaitResult};
use iceoryx2_pal_testing::assert_that;

const TIMEOUT: Duration = Duration::from_millis(25);

#[test]
fn semaphore_post_and_try_wait_works() {
    let initial_value = 5;
    let sut = Semaphore::new(initial_value);

    for _ in 0..initial_value {
        assert_that!(sut.try_wait(), eq WaitResult::Success);
    }
    assert_that!(sut.try_wait(), eq WaitResult::Interrupted);

    sut.post(|_| {}, initial_value);

    for _ in 0..initial_value {
        assert_that!(sut.try_wait(), eq WaitResult::Success);
    }
    assert_that!(sut.try_wait(), eq WaitResult::Interrupted);
}

#[test]
fn semaphore_post_and_wait_works() {
    let initial_value = 5;
    let sut = Semaphore::new(initial_value);

    for _ in 0..initial_value {
        assert_that!(sut.wait(|_, _| WaitAction::Abort), eq WaitResult::Success);
    }
    assert_that!(sut.wait(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);

    sut.post(|_| {}, initial_value);

    for _ in 0..initial_value {
        assert_that!(sut.wait(|_, _| WaitAction::Abort), eq WaitResult::Success);
    }
    assert_that!(sut.wait(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
}

#[test]
fn semaphore_wait_blocks() {
    let initial_value = 0;
    let sut = Semaphore::new(initial_value);
    let counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        s.spawn(|| {
            sut.wait(|_, _| WaitAction::Continue);
            counter.fetch_add(1, Ordering::Relaxed);
        });

        std::thread::sleep(TIMEOUT);
        let old_counter = counter.load(Ordering::Relaxed);
        sut.post(|_| {}, 1);

        assert_that!(old_counter, eq 0);
    });
}
