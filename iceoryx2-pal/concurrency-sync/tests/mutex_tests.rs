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
use std::time::Instant;

use iceoryx2_pal_concurrency_sync::{mutex::*, WaitAction, WaitResult};
use iceoryx2_pal_testing::assert_that;

const TIMEOUT: Duration = Duration::from_millis(25);

#[test]
fn mutex_lock_blocks() {
    let sut = Mutex::new();
    let counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        sut.try_lock();

        let t1 = s.spawn(|| {
            let lock_result = sut.lock(|_, _| WaitAction::Continue);
            assert_that!(lock_result, eq WaitResult::Success);
            counter.fetch_add(1, Ordering::Relaxed);
            sut.unlock(|_| {});
        });

        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        sut.unlock(|_| {});

        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    });
}

#[test]
fn mutex_lock_with_timeout_and_fails_after_timeout() {
    let sut = Mutex::new();

    sut.try_lock();

    assert_that!(sut.lock(|atomic, value| {
        let start = Instant::now();
        while atomic.load(Ordering::Relaxed) == *value {
            if start.elapsed() > TIMEOUT {
                return WaitAction::Abort;
            }
        }

        WaitAction::Continue
    }), eq WaitResult::Interrupted);
}
