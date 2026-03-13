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

use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::{AtomicU32, Ordering};
use iceoryx2_bb_concurrency::internal::strategy::semaphore::*;
use iceoryx2_bb_concurrency::{WaitAction, WaitResult};
use iceoryx2_bb_posix::{clock::nanosleep, thread::thread_scope};
use iceoryx2_bb_testing::assert_that;

pub fn strategy_semaphore_post_and_try_wait_works() {
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

pub fn strategy_semaphore_post_and_wait_works() {
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

pub fn strategy_semaphore_wait_blocks() {
    const TIMEOUT: Duration = Duration::from_millis(25);

    let initial_value = 0;
    let sut = Semaphore::new(initial_value);
    let counter = AtomicU32::new(0);

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                sut.wait(|_, _| WaitAction::Continue);
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        nanosleep(TIMEOUT).unwrap();
        let old_counter = counter.load(Ordering::Relaxed);
        sut.post(|_| {}, 1);

        assert_that!(old_counter, eq 0);

        Ok(())
    })
    .expect("failed to spawn thread");
}
