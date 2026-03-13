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
use iceoryx2_bb_concurrency::internal::strategy::mutex::*;
use iceoryx2_bb_concurrency::{WaitAction, WaitResult};
use iceoryx2_bb_posix::clock::{nanosleep, Time};
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;

pub const TIMEOUT: Duration = Duration::from_millis(25);

pub fn strategy_mutex_lock_blocks() {
    let sut = Mutex::new();
    let counter = AtomicU32::new(0);

    thread_scope(|s| {
        sut.try_lock();

        s.thread_builder()
            .spawn(|| {
                let lock_result = sut.lock(|_, _| WaitAction::Continue);
                assert_that!(lock_result, eq WaitResult::Success);
                counter.fetch_add(1, Ordering::Relaxed);
                sut.unlock(|_| {});
            })
            .expect("failed to spawn thread");

        nanosleep(TIMEOUT).unwrap();
        let counter_old = counter.load(Ordering::Relaxed);
        sut.unlock(|_| {});

        assert_that!(counter_old, eq 0);

        Ok(())
    })
    .expect("failed to spawn thread");

    assert_that!(counter.load(Ordering::Relaxed), eq 1);
}

pub fn strategy_mutex_lock_with_timeout_and_fails_after_timeout() {
    const TIMEOUT: Duration = Duration::from_millis(25);

    let sut = Mutex::new();

    sut.try_lock();

    assert_that!(sut.lock(|atomic, value| {
        let start = Time::now().expect("failure retrieving current time");
        while atomic.load(Ordering::Relaxed) == *value {
            if start.elapsed().expect("failed to get elapsed time") > TIMEOUT {
                return WaitAction::Abort;
            }
        }

        WaitAction::Continue
    }), eq WaitResult::Interrupted);
}
