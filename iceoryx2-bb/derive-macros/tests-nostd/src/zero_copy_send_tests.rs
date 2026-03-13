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

use iceoryx2_bb_derive_macros_tests_common::zero_copy_send_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn zero_copy_send_derive_works_for_named_struct() {
    zero_copy_send_tests::zero_copy_send_derive_works_for_named_struct();
}

#[inventory_test]
fn zero_copy_send_derive_works_for_unnamed_struct() {
    zero_copy_send_tests::zero_copy_send_derive_works_for_unnamed_struct();
}

#[inventory_test]
fn zero_copy_send_derive_works_for_generic_named_struct() {
    zero_copy_send_tests::zero_copy_send_derive_works_for_generic_named_struct();
}

#[inventory_test]
fn zero_copy_send_derive_works_for_generic_unnamed_struct() {
    zero_copy_send_tests::zero_copy_send_derive_works_for_generic_unnamed_struct();
}

#[inventory_test]
fn zero_copy_send_derive_sets_type_name_correctly_for_named_structs() {
    zero_copy_send_tests::zero_copy_send_derive_sets_type_name_correctly_for_named_structs();
}

#[inventory_test]
fn zero_copy_send_derive_sets_type_name_correctly_for_unnamed_structs() {
    zero_copy_send_tests::zero_copy_send_derive_sets_type_name_correctly_for_unnamed_structs();
}

#[inventory_test]
fn zero_copy_send_derive_sets_type_name_correctly_for_generic_named_structs() {
    zero_copy_send_tests::zero_copy_send_derive_sets_type_name_correctly_for_generic_named_structs(
    );
}

#[inventory_test]
fn zero_copy_send_derive_sets_type_name_correctly_for_generic_unnamed_struct() {
    zero_copy_send_tests::zero_copy_send_derive_sets_type_name_correctly_for_generic_unnamed_struct(
    );
}

#[inventory_test]
fn zero_copy_send_derive_for_unions() {
    zero_copy_send_tests::zero_copy_send_derive_for_unions();
}
