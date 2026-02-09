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

use iceoryx2_pal_testing_nostd_macros::requires_std;

#[requires_std("threading")]
pub fn strategy_mutex_lock_blocks() {
    use core::time::Duration;

    use iceoryx2_bb_concurrency::atomic::{AtomicU32, Ordering};
    use iceoryx2_bb_concurrency::internal::strategy::mutex::*;
    use iceoryx2_bb_concurrency::{WaitAction, WaitResult};
    use iceoryx2_pal_testing::assert_that;

    const TIMEOUT: Duration = Duration::from_millis(25);

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

#[requires_std("time")]
pub fn strategy_mutex_lock_with_timeout_and_fails_after_timeout() {
    use core::time::Duration;

    use iceoryx2_bb_concurrency::atomic::Ordering;
    use iceoryx2_bb_concurrency::internal::strategy::mutex::*;
    use iceoryx2_bb_concurrency::{WaitAction, WaitResult};
    use iceoryx2_pal_testing::assert_that;

    const TIMEOUT: Duration = Duration::from_millis(25);

    let sut = Mutex::new();

    sut.try_lock();

    assert_that!(sut.lock(|atomic, value| {
        let start = std::time::Instant::now();
        while atomic.load(Ordering::Relaxed) == *value {
            if start.elapsed() > TIMEOUT {
                return WaitAction::Abort;
            }
        }

        WaitAction::Continue
    }), eq WaitResult::Interrupted);
}
