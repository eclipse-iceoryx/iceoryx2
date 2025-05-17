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

use core::sync::atomic::{AtomicI32, Ordering};

use iceoryx2_pal_concurrency_sync::barrier::*;
use iceoryx2_pal_testing::assert_that;

#[test]
fn barrier_with_multiple_waiter_works() {
    let counter = AtomicI32::new(0);
    let sut = Barrier::new(4);
    let sut2 = Barrier::new(4);
    let sut3 = Barrier::new(4);

    std::thread::scope(|s| {
        s.spawn(|| {
            sut.wait(|_, _| {}, |_| {});
            sut2.wait(|_, _| {}, |_| {});
            counter.fetch_add(1, Ordering::Relaxed);
            sut3.wait(|_, _| {}, |_| {});
        });

        s.spawn(|| {
            sut.wait(|_, _| {}, |_| {});
            sut2.wait(|_, _| {}, |_| {});
            counter.fetch_add(1, Ordering::Relaxed);
            sut3.wait(|_, _| {}, |_| {});
        });

        s.spawn(|| {
            sut.wait(|_, _| {}, |_| {});
            sut2.wait(|_, _| {}, |_| {});
            counter.fetch_add(1, Ordering::Relaxed);
            sut3.wait(|_, _| {}, |_| {});
        });

        sut.wait(|_, _| {}, |_| {});
        let counter_old = counter.load(Ordering::Relaxed);
        sut2.wait(|_, _| {}, |_| {});

        sut3.wait(|_, _| {}, |_| {});

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 3);
    });
}
