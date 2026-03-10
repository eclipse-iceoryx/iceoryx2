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

extern crate iceoryx2_bb_loggers;

use enum_iterator::all;
use iceoryx2_bb_posix::{
    signal::{FetchableSignal, Signal},
    signal_set::{FetchableSignalSet, SignalSet},
};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING;

#[test]
fn new_empty_signal_set_does_not_contain_a_signal() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let sut = SignalSet::new_empty();

    for signal in all::<Signal>().collect::<Vec<Signal>>() {
        assert_that!(sut.contains(signal), eq false);
    }
}

#[test]
fn new_filled_signal_set_does_contain_all_signals() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let sut = SignalSet::new_filled();

    for signal in all::<Signal>().collect::<Vec<Signal>>() {
        assert_that!(sut.contains(signal), eq true);
    }
}

#[test]
fn adding_new_signals_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let mut sut = SignalSet::new_empty();

    for signal in all::<Signal>().collect::<Vec<Signal>>() {
        assert_that!(sut.contains(signal), eq false);
        sut.add(signal);
        assert_that!(sut.contains(signal), eq true);
    }
}

#[test]
fn removing_signals_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let mut sut = SignalSet::new_filled();

    for signal in all::<Signal>().collect::<Vec<Signal>>() {
        assert_that!(sut.contains(signal), eq true);
        sut.remove(signal);
        assert_that!(sut.contains(signal), eq false);
    }
}

#[test]
fn create_from_pending_signals_with_no_pending_signals_is_empty() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let sut = SignalSet::from_pending();

    for signal in all::<Signal>().collect::<Vec<Signal>>() {
        assert_that!(sut.contains(signal), eq false);
    }
}

#[test]
fn new_empty_fetchable_signal_set_does_not_contain_a_signal() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let sut = FetchableSignalSet::new_empty();

    for signal in all::<FetchableSignal>().collect::<Vec<FetchableSignal>>() {
        assert_that!(sut.contains(signal), eq false);
    }
}

#[test]
fn new_filled_fetchable_signal_set_does_contain_all_signals() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let sut = FetchableSignalSet::new_filled();

    for signal in all::<FetchableSignal>().collect::<Vec<FetchableSignal>>() {
        assert_that!(sut.contains(signal), eq true);
    }
}

#[test]
fn adding_new_fetchable_signals_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let mut sut = FetchableSignalSet::new_empty();

    for signal in all::<FetchableSignal>().collect::<Vec<FetchableSignal>>() {
        assert_that!(sut.contains(signal), eq false);
        sut.add(signal);
        assert_that!(sut.contains(signal), eq true);
    }
}

#[test]
fn removing_fetchable_signals_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let mut sut = FetchableSignalSet::new_filled();

    for signal in all::<FetchableSignal>().collect::<Vec<FetchableSignal>>() {
        assert_that!(sut.contains(signal), eq true);
        sut.remove(signal);
        assert_that!(sut.contains(signal), eq false);
    }
}

#[test]
fn create_from_pending_fetchable_signals_works() {
    test_requires!(POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);
    let sut = FetchableSignalSet::from_pending();

    for signal in all::<FetchableSignal>().collect::<Vec<FetchableSignal>>() {
        assert_that!(sut.contains(signal), eq false);
    }
}
