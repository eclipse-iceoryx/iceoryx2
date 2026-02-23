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

use iceoryx2_bb_elementary_tests_common::scope_guard_tests;

#[test]
pub fn scope_guard_callbacks_are_called_correctly_success_case() {
    scope_guard_tests::scope_guard_callbacks_are_called_correctly_success_case();
}

#[test]
pub fn scope_guard_callbacks_are_called_correctly_failure_case() {
    scope_guard_tests::scope_guard_callbacks_are_called_correctly_failure_case();
}
