// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_derive_macros_tests_common::placement_default_tests;

#[test]
fn placement_default_derive_for_structs_works() {
    placement_default_tests::placement_default_derive_for_structs_works();
}

#[test]
fn placement_default_derive_for_unnamed_structs_works() {
    placement_default_tests::placement_default_derive_for_unnamed_structs_works();
}

#[test]
fn placement_default_derive_for_generic_structs_works() {
    placement_default_tests::placement_default_derive_for_generic_structs_works();
}

#[test]
fn placement_default_derive_for_generic_unnamed_structs_works() {
    placement_default_tests::placement_default_derive_for_generic_unnamed_structs_works();
}
