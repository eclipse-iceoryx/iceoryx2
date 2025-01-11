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
use iceoryx2_bb_posix::adaptive_wait::*;
use iceoryx2_bb_posix::clock::*;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_testing::assert_that;
use std::time::Instant;

const TIMEOUT: Duration = Duration::from_millis(50);

#[test]
fn adaptive_wait_wait_at_different_time_depends_on_repetition_times() {
    let mut counter: u64 = 0;

    let mut waiter = AdaptiveWaitBuilder::new().create().unwrap();

    waiter
        .wait_while(move || -> bool {
            counter += 1;
            counter <= ADAPTIVE_WAIT_YIELD_REPETITIONS
        })
        .expect("failed to test wait_loop");
    // the waiter starts to sleep ADAPTIVE_WAIT_INITIAL_WAITING_TIME instead of yield later.
    assert_that!(waiter.yield_count(), eq ADAPTIVE_WAIT_YIELD_REPETITIONS);

    // test sleep time ADAPTIVE_WAIT_INITIAL_WAITING_TIME
    let start = Instant::now();
    waiter
        .wait_while(move || -> bool {
            counter += 1;
            counter <= 1 // stop at the second turn
        })
        .expect("failed to test wait_loop");
    assert_that!(start.elapsed(), time_at_least ADAPTIVE_WAIT_INITIAL_WAITING_TIME);

    waiter
        .wait_while(move || -> bool {
            counter += 1;
            // continue to reach the edge of sleeping time.
            counter < ADAPTIVE_WAIT_INITIAL_REPETITIONS - ADAPTIVE_WAIT_YIELD_REPETITIONS
        })
        .expect("failed to test wait_loop");

    // verify the waiter will enter the next stage at the next repetition.
    // the waiter starts to sleep longer as ADAPTIVE_WAIT_FINAL_WAITING_TIME
    // instead of ADAPTIVE_WAIT_INITIAL_WAITING_TIME later.
    assert_that!(waiter.yield_count(), eq ADAPTIVE_WAIT_INITIAL_REPETITIONS);
    let start = Instant::now();
    waiter
        .wait_while(move || -> bool {
            counter += 1;
            counter <= 1 // stop at the second turn
        })
        .expect("failed to test wait_loop");
    assert_that!(start.elapsed(), time_at_least ADAPTIVE_WAIT_FINAL_WAITING_TIME);
}

#[test]
fn adaptive_wait_on_default_builder_uses_default_clock() {
    let sut = AdaptiveWaitBuilder::new().create().unwrap();
    assert_that!(sut.clock_type(), eq ClockType::default());
}

#[test]
fn adaptive_wait_custom_clock_is_set_correctly() {
    let sut = AdaptiveWaitBuilder::new()
        .clock_type(ClockType::Realtime)
        .create()
        .unwrap();
    assert_that!(sut.clock_type(), eq ClockType::Realtime);
}

#[test]
fn adaptive_wait_wait_increases_yield_counter() {
    let mut sut = AdaptiveWaitBuilder::new().create().unwrap();
    assert_that!(sut.wait(), is_ok);
    assert_that!(sut.wait(), is_ok);
    assert_that!(sut.wait(), is_ok);
    assert_that!(sut.yield_count(), eq 3);
}

#[test]
fn adaptive_wait_timed_wait_while_wait_at_least_for_timeout() {
    let mut sut = AdaptiveWaitBuilder::new().create().unwrap();
    let start = Instant::now();

    let result = sut
        .timed_wait_while(|| -> Result<bool, ()> { Ok(true) }, TIMEOUT)
        .unwrap();

    assert_that!(start.elapsed(), time_at_least TIMEOUT);
    assert_that!(result, eq false);
}

#[test]
fn adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_false() {
    let mut sut = AdaptiveWaitBuilder::new().create().unwrap();
    let start = Instant::now();

    let result = sut
        .timed_wait_while(|| -> Result<bool, ()> { Ok(false) }, TIMEOUT)
        .unwrap();

    assert_that!(start.elapsed(), lt TIMEOUT);
    assert_that!(result, eq true);
}

#[test]
fn adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_error() {
    let mut sut = AdaptiveWaitBuilder::new().create().unwrap();
    let start = Instant::now();

    let result = sut.timed_wait_while(|| -> Result<bool, i32> { Err(5) }, TIMEOUT);

    assert_that!(start.elapsed(), lt TIMEOUT);
    assert_that!(result, is_err);
    assert_that!(
        result.err().unwrap(), eq
        AdaptiveTimedWaitWhileError::<i32>::PredicateFailure(5)
    );
}
