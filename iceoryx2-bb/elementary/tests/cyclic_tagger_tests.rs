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

use iceoryx2_bb_elementary_tests_common::cyclic_tagger_tests;

#[test]
pub fn create_tag_works() {
    cyclic_tagger_tests::create_tag_works();
}

#[test]
pub fn create_untagged_tag_works() {
    cyclic_tagger_tests::create_untagged_tag_works();
}

#[test]
pub fn tagging_after_new_cyclic_works() {
    cyclic_tagger_tests::tagging_after_new_cyclic_works();
}
