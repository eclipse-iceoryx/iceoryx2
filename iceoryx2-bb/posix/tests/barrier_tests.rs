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

use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_testing::assert_that;

use core::{sync::atomic::AtomicU64, sync::atomic::Ordering};
use std::thread;

#[test]
fn barrier_blocks() -> Result<(), BarrierCreationError> {
    let handle = BarrierHandle::new();
    let handle2 = BarrierHandle::new();
    let handle3 = BarrierHandle::new();
    let sut = BarrierBuilder::new(3).create(&handle)?;
    let sut2 = BarrierBuilder::new(3).create(&handle2)?;
    let sut3 = BarrierBuilder::new(3).create(&handle3)?;
    let counter = AtomicU64::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            sut.wait();
            sut2.wait();
            counter.fetch_add(10, Ordering::Relaxed);
            sut3.wait();
        });
        s.spawn(|| {
            sut.wait();
            sut2.wait();
            counter.fetch_add(10, Ordering::Relaxed);
            sut3.wait();
        });

        sut.wait();
        let counter_old = counter.load(Ordering::Relaxed);
        sut2.wait();
        sut3.wait();

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 20);
    });

    Ok(())
}
