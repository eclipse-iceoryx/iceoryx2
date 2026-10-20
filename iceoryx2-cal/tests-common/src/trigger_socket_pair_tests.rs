// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_cal::trigger::socket_pair::SocketPair;
use iceoryx2_cal::trigger::*;

#[test]
fn activating_a_trigger_without_observer_fails() {
    let (activator, observer) = <SocketPair as Trigger>::Builder::new().create().unwrap();

    drop(observer);

    assert_that!(activator.activate().err(), eq Some(TriggerActivateError::Disconnected));
}

#[test]
fn activating_a_trigger_with_abandoned_observer_fails() {
    let (activator, observer) = <SocketPair as Trigger>::Builder::new().create().unwrap();

    observer.abandon();

    assert_that!(activator.activate().err(), eq Some(TriggerActivateError::Disconnected));
}

#[test]
fn resetting_a_trigger_with_abandoned_activator_reports_no_activation() {
    let (activator, observer) = <SocketPair as Trigger>::Builder::new().create().unwrap();

    activator.abandon();

    assert_that!(observer.reset(), eq Ok(false));
}
