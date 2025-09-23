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

use std::sync::{atomic::Ordering, Barrier};

use iceoryx2_bb_linux::signalfd::SignalFdBuilder;
use iceoryx2_bb_posix::{
    process::Process,
    signal::{FetchableSignal, SignalHandler},
    signal_set::FetchableSignalSet,
    user::User,
};
use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

#[test]
fn registered_signal_can_be_try_read() {
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined1);
    let sut = SignalFdBuilder::new(signals).create_non_blocking().unwrap();
    SignalHandler::call_and_fetch(|| {
        Process::from_self()
            .send_signal(FetchableSignal::UserDefined1.into())
            .unwrap();
    });

    let signal = sut.try_read().unwrap().unwrap();
    assert_that!(signal.signal(), eq FetchableSignal::UserDefined1);
    assert_that!(signal.origin_pid(), eq Process::from_self().id());
    assert_that!(signal.origin_uid(), eq User::from_self().unwrap().uid());
}

#[test]
fn without_signal_try_read_returns_none() {
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined1);
    let sut = SignalFdBuilder::new(signals).create_non_blocking().unwrap();

    assert_that!(sut.try_read().unwrap(), is_none);
}

#[test]
fn registered_signal_can_be_blocking_read() {
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined2);
    let sut = SignalFdBuilder::new(signals).create_blocking().unwrap();
    SignalHandler::call_and_fetch(|| {
        Process::from_self()
            .send_signal(FetchableSignal::UserDefined2.into())
            .unwrap();
    });

    let signal = sut.blocking_read().unwrap().unwrap();
    assert_that!(signal.signal(), eq FetchableSignal::UserDefined2);
    assert_that!(signal.origin_pid(), eq Process::from_self().id());
    assert_that!(signal.origin_uid(), eq User::from_self().unwrap().uid());
}

#[test]
fn blocking_read_blocks() {
    let _watchdog = Watchdog::new();
    let counter = IoxAtomicU64::new(0);
    let barrier = Barrier::new(2);
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined2);
    let sut = SignalFdBuilder::new(signals).create_blocking().unwrap();

    std::thread::scope(|s| {
        s.spawn(|| {
            barrier.wait();

            let signal = sut.blocking_read().unwrap().unwrap();
            assert_that!(signal.signal(), eq FetchableSignal::UserDefined2);
            assert_that!(signal.origin_pid(), eq Process::from_self().id());
            assert_that!(signal.origin_uid(), eq User::from_self().unwrap().uid());
            counter.store(1, Ordering::Relaxed);
        });

        barrier.wait();
        std::thread::sleep(core::time::Duration::from_millis(50));
        assert_that!(counter.load(Ordering::Relaxed), eq 0);

        SignalHandler::call_and_fetch(|| {
            Process::from_self()
                .send_signal(FetchableSignal::UserDefined2.into())
                .unwrap();
        });
    });
}
