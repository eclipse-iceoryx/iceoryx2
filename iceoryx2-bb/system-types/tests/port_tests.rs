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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_system_types_tests_common::port_tests;

#[test]
fn port_set_works() {
    port_tests::port_set_works();
}

#[test]
fn port_is_unspecified_works() {
    port_tests::port_is_unspecified_works();
}

#[test]
fn port_is_system_works() {
    port_tests::port_is_system_works();
}

#[test]
fn port_is_registered_works() {
    port_tests::port_is_registered_works();
}

#[test]
fn port_is_dynamic_works() {
    port_tests::port_is_dynamic_works();
}

#[test]
fn port_try_from_str_work() {
    port_tests::port_try_from_str_work();
}

#[test]
fn port_try_from_str_with_invalid_integer_fails() {
    port_tests::port_try_from_str_with_invalid_integer_fails();
}
