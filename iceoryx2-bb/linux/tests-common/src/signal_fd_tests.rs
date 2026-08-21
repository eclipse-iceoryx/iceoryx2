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

use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};
use iceoryx2_bb_linux::signalfd::SignalFdBuilder;
use iceoryx2_bb_posix::barrier::BarrierBuilder;
use iceoryx2_bb_posix::barrier::BarrierHandle;
use iceoryx2_bb_posix::barrier::Handle;
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_posix::{
    process::Process,
    signal::{FetchableSignal, NonFatalFetchableSignal, SignalHandler},
    signal_set::FetchableSignalSet,
    user::User,
};
use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};
use iceoryx2_bb_testing_macros::test;

// A signal subscribed on a signalfd must be delivered to the fd. Today the
// global SignalHandler claims every non-fatal signal first and the fd is
// starved.
// TODO: #1898
#[ignore]
#[test]
fn registered_signal_can_be_try_read() {
    let _watchdog = Watchdog::new();
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined1);
    let sut = SignalFdBuilder::new(signals).create_non_blocking().unwrap();

    // activate the global handler, currently starving the fd
    let _ = SignalHandler::call_and_fetch(|| {});

    SignalHandler::call_and_fetch(|| {
        Process::from_self()
            .send_signal(FetchableSignal::UserDefined1.into())
            .unwrap();
    });

    let mut received = None;
    for _ in 0..100 {
        if let Some(signal) = sut.try_read().unwrap() {
            received = Some(signal);
            break;
        }
        nanosleep(core::time::Duration::from_millis(2)).ok();
    }

    let received = received.unwrap();
    assert_that!(received.signal(), eq FetchableSignal::UserDefined1);
    assert_that!(received.origin_pid(), eq Process::from_self().id());
    assert_that!(received.origin_uid(), eq User::from_self().unwrap().uid());
}

// Regression guard for #1898: dropping a SignalFd that subscribed to
// UserDefined1 must not leave that signal monopolized away from the global
// handler. After the fd is released the handler must observe UserDefined1
// again.
#[test]
fn dropped_signal_fd_restores_handler_visibility() {
    let _watchdog = Watchdog::new();
    let _ = SignalHandler::call_and_fetch(|| {});

    {
        let mut signals = FetchableSignalSet::new_empty();
        signals.add(FetchableSignal::UserDefined1);
        let _sut = SignalFdBuilder::new(signals).create_non_blocking().unwrap();
    } // fd dropped here

    let observed = SignalHandler::call_and_fetch(|| {
        Process::from_self()
            .send_signal(FetchableSignal::UserDefined1.into())
            .unwrap();
        nanosleep(core::time::Duration::from_millis(2)).ok();
    });

    assert_that!(observed, eq Some(NonFatalFetchableSignal::UserDefined1));
}

// Regression guard for #1898: while a SignalFd owns UserDefined1 the
// global handler must still observe an unrelated signal (Continue). Owning one
// signal must not suppress delivery of a different, unowned signal.
#[test]
fn signal_fd_does_not_mask_unsubscribed_signal() {
    let _watchdog = Watchdog::new();
    let _ = SignalHandler::call_and_fetch(|| {});

    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined1);
    let _sut = SignalFdBuilder::new(signals).create_non_blocking().unwrap();

    let observed = SignalHandler::call_and_fetch(|| {
        Process::from_self()
            .send_signal(FetchableSignal::Continue.into())
            .unwrap();
        nanosleep(core::time::Duration::from_millis(2)).ok();
    });

    assert_that!(observed, eq Some(NonFatalFetchableSignal::Continue));
}

#[test]
fn without_signal_try_read_returns_none() {
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined1);
    let sut = SignalFdBuilder::new(signals).create_non_blocking().unwrap();

    assert_that!(sut.try_read().unwrap(), is_none);
}

#[test]
fn blocking_read_blocks() {
    let _watchdog = Watchdog::new();
    let counter = AtomicU64::new(0);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let mut signals = FetchableSignalSet::new_empty();
    signals.add(FetchableSignal::UserDefined2);
    let sut = SignalFdBuilder::new(signals).create_blocking().unwrap();

    thread_scope(|s| {
        s.thread_builder().spawn(|| {
            barrier.wait();

            let signal = sut.blocking_read().unwrap().unwrap();
            assert_that!(signal.signal(), eq FetchableSignal::UserDefined2);
            assert_that!(signal.origin_pid(), eq Process::from_self().id());
            assert_that!(signal.origin_uid(), eq User::from_self().unwrap().uid());
            counter.store(1, Ordering::Relaxed);
        })?;

        barrier.wait();
        nanosleep(core::time::Duration::from_millis(50)).unwrap();
        assert_that!(counter.load(Ordering::Relaxed), eq 0);

        while counter.load(Ordering::Relaxed) == 0 {
            SignalHandler::call_and_fetch(|| {
                Process::from_self()
                    .send_signal(FetchableSignal::UserDefined2.into())
                    .unwrap();
            });
        }

        Ok(())
    })
    .unwrap();
}
