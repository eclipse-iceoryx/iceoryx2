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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::polymorphic_vec_tests;

#[test]
fn try_clone_clones_empty_vec() {
    polymorphic_vec_tests::try_clone_clones_empty_vec();
}

#[test]
fn try_clone_clones_filled_vec() {
    polymorphic_vec_tests::try_clone_clones_filled_vec();
}

#[test]
fn two_vectors_with_same_content_are_equal() {
    polymorphic_vec_tests::two_vectors_with_same_content_are_equal();
}

#[test]
fn two_vectors_with_different_content_are_not_equal() {
    polymorphic_vec_tests::two_vectors_with_different_content_are_not_equal();
}

#[test]
fn two_vectors_with_different_len_are_not_equal() {
    polymorphic_vec_tests::two_vectors_with_different_len_are_not_equal();
}

#[test]
fn from_fn_initializes_vector() {
    polymorphic_vec_tests::from_fn_initializes_vector();
}
