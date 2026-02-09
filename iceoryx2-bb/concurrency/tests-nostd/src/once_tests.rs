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

#![allow(clippy::disallowed_types)]

use iceoryx2_bb_concurrency_tests_common::once_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn once_executes_exactly_once() {
    once_tests::once_executes_exactly_once();
}

#[inventory_test]
fn once_works_with_multiple_threads() {
    once_tests::once_works_with_multiple_threads();
}

#[inventory_test]
fn once_is_completed_returns_false_initially() {
    once_tests::once_is_completed_returns_false_initially();
}
