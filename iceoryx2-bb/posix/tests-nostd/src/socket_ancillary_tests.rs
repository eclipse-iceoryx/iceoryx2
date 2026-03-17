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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::socket_ancillary_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn socket_ancillary_is_empty_when_created() {
    socket_ancillary_tests::socket_ancillary_is_empty_when_created();
}

#[inventory_test]
fn socket_ancillary_credentials_work() {
    socket_ancillary_tests::socket_ancillary_credentials_work();
}

#[inventory_test]
fn socket_ancillary_add_file_descriptors_work() {
    socket_ancillary_tests::socket_ancillary_add_file_descriptors_work();
}
