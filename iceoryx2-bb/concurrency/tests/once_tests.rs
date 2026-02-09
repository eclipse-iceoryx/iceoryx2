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

use iceoryx2_bb_concurrency_tests_common::once_tests;

#[test]
fn once_executes_exactly_once() {
    once_tests::once_executes_exactly_once();
}

#[test]
fn once_works_with_multiple_threads() {
    once_tests::once_works_with_multiple_threads();
}

#[test]
fn once_is_completed_returns_false_initially() {
    once_tests::once_is_completed_returns_false_initially();
}
