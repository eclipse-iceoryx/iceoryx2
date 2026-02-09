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

use iceoryx2_bb_concurrency_tests_common::strategy_condition_variable_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn strategy_condition_variable_notify_one_unblocks_one() {
    strategy_condition_variable_tests::strategy_condition_variable_notify_one_unblocks_one();
}

#[inventory_test]
fn strategy_condition_variable_notify_all_unblocks_all() {
    strategy_condition_variable_tests::strategy_condition_variable_notify_all_unblocks_all();
}

#[inventory_test]
fn strategy_condition_variable_mutex_is_locked_when_wait_returns() {
    strategy_condition_variable_tests::strategy_condition_variable_mutex_is_locked_when_wait_returns();
}

#[inventory_test]
fn strategy_condition_variable_wait_returns_false_when_functor_returns_false() {
    strategy_condition_variable_tests::strategy_condition_variable_wait_returns_false_when_functor_returns_false();
}
