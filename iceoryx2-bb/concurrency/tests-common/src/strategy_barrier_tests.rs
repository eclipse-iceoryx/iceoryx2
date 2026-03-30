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

use iceoryx2_bb_concurrency::atomic::{AtomicI32, Ordering};
use iceoryx2_bb_concurrency::internal::strategy::barrier::*;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn multiple_waiters_works() {
    let counter = AtomicI32::new(0);
    let sut = Barrier::new(4);
    let sut2 = Barrier::new(4);
    let sut3 = Barrier::new(4);

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                sut.wait(|_, _| {}, |_| {});
                sut2.wait(|_, _| {}, |_| {});
                counter.fetch_add(1, Ordering::Relaxed);
                sut3.wait(|_, _| {}, |_| {});
            })
            .expect("failed to spawn thread");

        s.thread_builder()
            .spawn(|| {
                sut.wait(|_, _| {}, |_| {});
                sut2.wait(|_, _| {}, |_| {});
                counter.fetch_add(1, Ordering::Relaxed);
                sut3.wait(|_, _| {}, |_| {});
            })
            .expect("failed to spawn thread");

        s.thread_builder()
            .spawn(|| {
                sut.wait(|_, _| {}, |_| {});
                sut2.wait(|_, _| {}, |_| {});
                counter.fetch_add(1, Ordering::Relaxed);
                sut3.wait(|_, _| {}, |_| {});
            })
            .expect("failed to spawn thread");

        sut.wait(|_, _| {}, |_| {});
        sut2.wait(|_, _| {}, |_| {});
        sut3.wait(|_, _| {}, |_| {});

        assert_that!(counter.load(Ordering::Relaxed), eq 3);

        Ok(())
    })
    .expect("failed to spawn thread");
}
