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

#![allow(clippy::disallowed_types)]

use iceoryx2_bb_concurrency_tests_common::lazy_lock_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn lazy_lock_primitive_type() {
    lazy_lock_tests::lazy_lock_primitive_type();
}

#[inventory_test]
fn lazy_lock_complex_type() {
    lazy_lock_tests::lazy_lock_complex_type();
}

#[inventory_test]
fn lazy_lock_zero_sized_type() {
    lazy_lock_tests::lazy_lock_zero_sized_type();
}

#[inventory_test]
fn lazy_lock_closure() {
    lazy_lock_tests::lazy_lock_closure();
}

#[inventory_test]
fn lazy_lock_non_static() {
    lazy_lock_tests::lazy_lock_non_static();
}

#[inventory_test]
fn lazy_lock_deref() {
    lazy_lock_tests::lazy_lock_deref();
}

#[inventory_test]
fn lazy_lock_initialization_occurs_once() {
    lazy_lock_tests::lazy_lock_initialization_occurs_once();
}

#[inventory_test]
fn lazy_lock_force_initialization() {
    lazy_lock_tests::lazy_lock_force_initialization();
}

#[inventory_test]
fn lazy_lock_returns_same_reference() {
    lazy_lock_tests::lazy_lock_returns_same_reference();
}

#[inventory_test]
fn lazy_lock_dependent_initialization() {
    lazy_lock_tests::lazy_lock_dependent_initialization();
}

#[inventory_test]
fn lazy_lock_access_concurrent_access_from_multiple_threads() {
    lazy_lock_tests::lazy_lock_access_concurrent_access_from_multiple_threads();
}
