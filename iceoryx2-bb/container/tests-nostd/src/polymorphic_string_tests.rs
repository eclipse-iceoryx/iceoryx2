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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::polymorphic_string_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn try_clone_clones_empty_string() {
    polymorphic_string_tests::try_clone_clones_empty_string();
}

#[inventory_test]
fn try_clone_clones_filled_string() {
    polymorphic_string_tests::try_clone_clones_filled_string();
}
