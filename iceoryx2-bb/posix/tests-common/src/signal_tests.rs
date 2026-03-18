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

use iceoryx2_bb_testing_macros::requires_std;

use iceoryx2_bb_testing_macros::inventory_test;
#[cfg(feature = "std")]
use std_testing::*;

#[cfg(feature = "std")]
mod std_testing {
    use core::time::Duration;
    use iceoryx2_bb_concurrency::atomic::AtomicUsize;
    use std::sync::Mutex;

    pub use alloc::vec;

    pub use iceoryx2_bb_concurrency::atomic::{AtomicI32, Ordering};
    pub use iceoryx2_bb_posix::clock::nanosleep;
    pub use iceoryx2_bb_posix::clock::ClockType;
    pub use iceoryx2_bb_posix::clock::Time;
    pub use iceoryx2_bb_posix::process::*;
    pub use iceoryx2_bb_posix::signal::*;
    pub use iceoryx2_bb_testing::assert_that;
    pub use iceoryx2_bb_testing::test_requires;
    pub use iceoryx2_bb_testing::watchdog::Watchdog;
    pub use iceoryx2_pal_posix::posix::support::POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING;
    pub use iceoryx2_pal_posix::*;

    pub static COUNTER: AtomicUsize = AtomicUsize::new(0);
    pub static SIGNAL: AtomicUsize = AtomicUsize::new(posix::MAX_SIGNAL_VALUE);
    pub static LOCK: Mutex<i32> = Mutex::new(0);
    pub const TIMEOUT: Duration = Duration::from_millis(100);
}

#[cfg(feature = "std")]
struct TestFixture {
    _guard: std::sync::MutexGuard<'static, i32>,
}

#[cfg(feature = "std")]
impl TestFixture {
    fn new() -> Self {
        let new_self = Self {
            _guard: LOCK.lock().unwrap(),
        };

        COUNTER.store(0, Ordering::SeqCst);
        SIGNAL.store(posix::MAX_SIGNAL_VALUE, Ordering::SeqCst);

        new_self
    }

    pub fn signal_callback(signal: FetchableSignal) {
        SIGNAL.store(signal as usize, Ordering::SeqCst);
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    pub fn verify(&self, signal: NonFatalFetchableSignal, counter_value: usize) {
        assert_that!(
            || { COUNTER.load(Ordering::SeqCst) },
            eq counter_value,
            before Watchdog::default()
        );

        assert_that!(SignalHandler::last_signal(), eq Some(signal));
        assert_that!(SIGNAL.load(Ordering::SeqCst), eq signal as usize);
    }
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_register_single_handler_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let test = TestFixture::new();
    let _guard =
        SignalHandler::register(FetchableSignal::UserDefined1, &TestFixture::signal_callback);

    Process::from_self().send_signal(Signal::UserDefined1).ok();
    test.verify(NonFatalFetchableSignal::UserDefined1, 1)
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_register_multiple_handler_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let test = TestFixture::new();
    let _guard1 =
        SignalHandler::register(FetchableSignal::UserDefined1, &TestFixture::signal_callback);

    let _guard2 =
        SignalHandler::register(FetchableSignal::UserDefined2, &TestFixture::signal_callback);

    Process::from_self().send_signal(Signal::UserDefined1).ok();
    test.verify(NonFatalFetchableSignal::UserDefined1, 1);

    Process::from_self().send_signal(Signal::UserDefined2).ok();
    test.verify(NonFatalFetchableSignal::UserDefined2, 2);
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_register_handler_with_multiple_signals_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let test = TestFixture::new();
    let s = vec![FetchableSignal::UserDefined1, FetchableSignal::UserDefined2];
    let _guard1 = SignalHandler::register_multiple_signals(&s, &TestFixture::signal_callback);

    Process::from_self().send_signal(Signal::UserDefined1).ok();
    test.verify(NonFatalFetchableSignal::UserDefined1, 1);

    Process::from_self().send_signal(Signal::UserDefined2).ok();
    test.verify(NonFatalFetchableSignal::UserDefined2, 2);
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_guard_unregisters_on_drop() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let test = TestFixture::new();
    let guard1 =
        SignalHandler::register(FetchableSignal::UserDefined1, &TestFixture::signal_callback);

    drop(guard1);

    let _guard1 = SignalHandler::register(FetchableSignal::UserDefined1, &|signal| {
        SIGNAL.store(signal as usize, Ordering::SeqCst);
        COUNTER.fetch_add(10, Ordering::SeqCst);
    });

    Process::from_self().send_signal(Signal::UserDefined1).ok();
    test.verify(NonFatalFetchableSignal::UserDefined1, 10);
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_register_signal_twice_fails() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let _test = TestFixture::new();
    let s = vec![FetchableSignal::UserDefined1, FetchableSignal::UserDefined2];
    let _guard1 = SignalHandler::register_multiple_signals(&s, &TestFixture::signal_callback);

    assert_that!(
        SignalHandler::register(FetchableSignal::UserDefined2, &TestFixture::signal_callback),
        is_err
    );
}

#[inventory_test]
#[requires_std("threading", "watchdog")]
pub fn signal_call_and_fetch_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let _watchdog = Watchdog::new();

    let _test = TestFixture::new();
    let result = SignalHandler::call_and_fetch(|| {
        Process::from_self().send_signal(Signal::Interrupt).ok();
        nanosleep(TIMEOUT).ok();
    });

    assert_that!(result, eq Some(NonFatalFetchableSignal::Interrupt));
}

#[inventory_test]
#[requires_std("threading", "watchdog")]
pub fn signal_call_and_fetch_with_registered_handler_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let _watchdog = Watchdog::new();

    let test = TestFixture::new();

    let _guard =
        SignalHandler::register(FetchableSignal::UserDefined1, &TestFixture::signal_callback);

    let result = SignalHandler::call_and_fetch(|| {
        Process::from_self().send_signal(Signal::UserDefined1).ok();
        nanosleep(TIMEOUT).ok();
    });

    assert_that!(result, eq Some(NonFatalFetchableSignal::UserDefined1));
    test.verify(NonFatalFetchableSignal::UserDefined1, 1);
}

#[inventory_test]
#[requires_std("threading", "watchdog")]
pub fn signal_wait_for_signal_blocks() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let _watchdog = Watchdog::new();

    let _test = TestFixture::new();

    let signals = vec![
        NonFatalFetchableSignal::UserDefined2,
        NonFatalFetchableSignal::UserDefined1,
    ];
    let counter = AtomicI32::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            SignalHandler::wait_for_multiple_signals(&signals).unwrap();
            counter.store(1, Ordering::Relaxed);
        });

        nanosleep(TIMEOUT).ok();
        let counter_old = counter.load(Ordering::Relaxed);
        Process::from_self().send_signal(Signal::UserDefined2).ok();

        assert_that!(counter_old, eq 0);
        assert_that!(
            || { counter.load(Ordering::Relaxed) },
            eq 1,
            before Watchdog::default()
        );
    });
}

#[inventory_test]
#[requires_std("threading", "watchdog")]
pub fn signal_wait_twice_for_same_signal_blocks() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let _watchdog = Watchdog::new();

    let _test = TestFixture::new();

    let counter = AtomicI32::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            SignalHandler::wait_for_signal(NonFatalFetchableSignal::UserDefined2).unwrap();
            counter.fetch_add(1, Ordering::Relaxed);
        });

        nanosleep(TIMEOUT).ok();
        let counter_old = counter.load(Ordering::Relaxed);
        Process::from_self().send_signal(Signal::UserDefined2).ok();

        s.spawn(|| {
            SignalHandler::wait_for_signal(NonFatalFetchableSignal::UserDefined2).unwrap();
            counter.fetch_add(1, Ordering::Relaxed);
        });

        nanosleep(TIMEOUT).ok();
        let counter_old_2 = counter.load(Ordering::Relaxed);
        Process::from_self().send_signal(Signal::UserDefined2).ok();

        assert_that!(counter_old, eq 0);
        assert_that!(counter_old_2, le 1);
        assert_that!(
            || { counter.load(Ordering::Relaxed) },
            eq 2,
            before Watchdog::default()
        );
    });
}

#[inventory_test]
#[requires_std("threading", "watchdog")]
pub fn signal_timed_wait_blocks_at_least_for_timeout() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let _watchdog = Watchdog::new();

    let _test = TestFixture::new();

    let start = Time::now_with_clock(ClockType::Monotonic).unwrap();
    SignalHandler::timed_wait_for_signal(NonFatalFetchableSignal::UserDefined2, TIMEOUT).unwrap();
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
}

#[inventory_test]
#[requires_std("threading", "watchdog")]
pub fn signal_timed_wait_blocks_until_signal() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let _watchdog = Watchdog::new();

    let _test = TestFixture::new();

    let signals = vec![
        NonFatalFetchableSignal::UserDefined2,
        NonFatalFetchableSignal::UserDefined1,
    ];
    let counter = AtomicI32::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            SignalHandler::timed_wait_for_multiple_signals(&signals, 100 * TIMEOUT).unwrap();
            counter.store(1, Ordering::Relaxed);
        });

        nanosleep(TIMEOUT).ok();
        let counter_old = counter.load(Ordering::Relaxed);
        Process::from_self().send_signal(Signal::UserDefined2).ok();

        assert_that!(counter_old, eq 0);
        assert_that!(
            || { counter.load(Ordering::Relaxed) },
            eq 1,
            before Watchdog::default()
        );
    });
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_termination_requested_with_terminate_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let _test = TestFixture::new();

    assert_that!(!SignalHandler::termination_requested(), eq true);
    assert_that!(Process::from_self().send_signal(Signal::Terminate), is_ok);

    assert_that!(
        || { SignalHandler::termination_requested() },
        eq true,
        before Watchdog::default()
    );
    assert_that!(SignalHandler::termination_requested(), eq false);
}

#[inventory_test]
#[requires_std("threading")]
pub fn signal_termination_requested_with_interrupt_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let _test = TestFixture::new();

    assert_that!(SignalHandler::termination_requested(), eq false);
    assert_that!(Process::from_self().send_signal(Signal::Interrupt), is_ok);

    assert_that!(
        || { SignalHandler::termination_requested() },
        eq true,
        before Watchdog::default()
    );
    assert_that!(SignalHandler::termination_requested(), eq false);
}
