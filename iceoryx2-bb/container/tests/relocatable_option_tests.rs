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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::relocatable_option_tests;

#[test]
fn default_created_option_is_empty() {
    relocatable_option_tests::default_created_option_is_empty();
}

#[test]
fn when_empty_as_option_returns_empty_option() {
    relocatable_option_tests::when_empty_as_option_returns_empty_option();
}

#[test]
fn when_with_value_as_option_returns_option_with_reference_to_that_value() {
    relocatable_option_tests::when_with_value_as_option_returns_option_with_reference_to_that_value(
    );
}

#[test]
fn when_empty_as_option_mut_returns_empty_option() {
    relocatable_option_tests::when_empty_as_option_mut_returns_empty_option();
}

#[test]
fn when_with_value_as_option_returns_option_with_mut_reference_to_that_value() {
    relocatable_option_tests::when_with_value_as_option_returns_option_with_mut_reference_to_that_value();
}

#[test]
fn when_empty_as_deref_returns_empty_option() {
    relocatable_option_tests::when_empty_as_deref_returns_empty_option();
}

#[test]
fn when_with_value_as_deref_returns_ref_to_target() {
    relocatable_option_tests::when_with_value_as_deref_returns_ref_to_target();
}

#[test]
fn when_empty_as_deref_mut_returns_empty_option() {
    relocatable_option_tests::when_empty_as_deref_mut_returns_empty_option();
}

#[test]
fn when_with_value_as_deref_mut_returns_ref_to_target() {
    relocatable_option_tests::when_with_value_as_deref_mut_returns_ref_to_target();
}

#[test]
fn when_empty_as_ref_returns_empty_option() {
    relocatable_option_tests::when_empty_as_ref_returns_empty_option();
}

#[test]
fn when_with_value_as_ref_returns_ref_to_target() {
    relocatable_option_tests::when_with_value_as_ref_returns_ref_to_target();
}

#[test]
fn when_empty_as_mut_returns_empty_option() {
    relocatable_option_tests::when_empty_as_mut_returns_empty_option();
}

#[test]
fn when_with_value_as_mut_returns_ref_to_target() {
    relocatable_option_tests::when_with_value_as_mut_returns_ref_to_target();
}

#[test]
fn expect_returns_value() {
    relocatable_option_tests::expect_returns_value();
}

#[should_panic]
#[test]
fn expect_panics_when_empty() {
    relocatable_option_tests::expect_panics_when_empty();
}

#[test]
fn none_creates_empty_value() {
    relocatable_option_tests::none_creates_empty_value();
}

#[test]
fn some_creates_option_that_contains_the_value() {
    relocatable_option_tests::some_creates_option_that_contains_the_value();
}

#[test]
fn inspect_callback_is_not_called_when_empty() {
    relocatable_option_tests::inspect_callback_is_not_called_when_empty();
}

#[test]
fn inspect_callback_is_called_when_it_contains_a_value() {
    relocatable_option_tests::inspect_callback_is_called_when_it_contains_a_value();
}

#[test]
fn map_of_empty_option_is_empty() {
    relocatable_option_tests::map_of_empty_option_is_empty();
}

#[test]
fn map_uses_value_and_creates_new_option() {
    relocatable_option_tests::map_uses_value_and_creates_new_option();
}

#[test]
fn replace_returns_none_when_option_is_empty() {
    relocatable_option_tests::replace_returns_none_when_option_is_empty();
}

#[test]
fn replace_returns_value_when_option_contains_value() {
    relocatable_option_tests::replace_returns_value_when_option_contains_value();
}

#[test]
fn take_empties_sut_and_returns_content() {
    relocatable_option_tests::take_empties_sut_and_returns_content();
}

#[test]
fn take_if_does_not_call_callback_when_empty() {
    relocatable_option_tests::take_if_does_not_call_callback_when_empty();
}

#[test]
fn take_if_returns_none_when_callback_returns_false() {
    relocatable_option_tests::take_if_returns_none_when_callback_returns_false();
}

#[test]
fn take_if_returns_value_and_empties_option_when_callback_returns_true() {
    relocatable_option_tests::take_if_returns_value_and_empties_option_when_callback_returns_true();
}

#[test]
fn unwrap_returns_value_when_it_has_one() {
    relocatable_option_tests::unwrap_returns_value_when_it_has_one();
}

#[should_panic]
#[test]
fn unwrap_panics_when_empty() {
    relocatable_option_tests::unwrap_panics_when_empty();
}

#[test]
fn unwrap_or_returns_provided_value_when_empty() {
    relocatable_option_tests::unwrap_or_returns_provided_value_when_empty();
}

#[test]
fn unwrap_or_returns_value() {
    relocatable_option_tests::unwrap_or_returns_value();
}

#[test]
fn unwrap_or_default_returns_default_when_empty() {
    relocatable_option_tests::unwrap_or_default_returns_default_when_empty();
}

#[test]
fn unwrap_or_default_returns_value() {
    relocatable_option_tests::unwrap_or_default_returns_value();
}

#[test]
fn unwrap_or_else_returns_callable_value_when_empty() {
    relocatable_option_tests::unwrap_or_else_returns_callable_value_when_empty();
}

#[test]
fn unwrap_or_else_returns_value() {
    relocatable_option_tests::unwrap_or_else_returns_value();
}

#[test]
fn unwrap_unchecked_returns_value() {
    relocatable_option_tests::unwrap_unchecked_returns_value();
}

#[test]
fn element_is_dropped_on_option_drop() {
    relocatable_option_tests::element_is_dropped_on_option_drop();
}

#[test]
fn clone_works() {
    relocatable_option_tests::clone_works();
}

#[test]
fn placement_default_works() {
    relocatable_option_tests::placement_default_works();
}

#[test]
fn serialization_works() {
    relocatable_option_tests::serialization_works();
}

#[test]
fn empty_create_empty_native_option() {
    relocatable_option_tests::empty_create_empty_native_option();
}

#[test]
fn value_creates_native_option_with_value() {
    relocatable_option_tests::value_creates_native_option_with_value();
}

#[test]
fn native_to_relocatable_conversion_works() {
    relocatable_option_tests::native_to_relocatable_conversion_works();
}

#[test]
fn relocatable_to_native_conversion_works() {
    relocatable_option_tests::relocatable_to_native_conversion_works();
}
