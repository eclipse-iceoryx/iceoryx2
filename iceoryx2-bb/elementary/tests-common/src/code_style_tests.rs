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

use iceoryx2_bb_elementary::code_style::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn snake_case_conversion_works() {
    assert_that!(
        camel_to_snake_case("the_toad_is_watching_you"), eq
        "the_toad_is_watching_you"
    );

    assert_that!(
        camel_to_snake_case("BiteDust"), eq
        "bite_dust"
    );

    assert_that!(
        camel_to_snake_case("LickAToad"), eq
        "lick_atoad"
    );

    assert_that!(
        camel_to_snake_case("here__is__nothing__ToSniff"), eq
        "here__is__nothing__to_sniff"
    );

    assert_that!(
        camel_to_snake_case("A"), eq
        "a"
    );

    assert_that!(
        camel_to_snake_case("?"), eq
        "?"
    );

    assert_that!(
        camel_to_snake_case("_"), eq
        "_"
    );

    assert_that!(
        camel_to_snake_case("a"), eq
        "a"
    );

    assert_that!(
        camel_to_snake_case("_wherever_you_go"), eq
        "_wherever_you_go"
    );

    assert_that!(
        camel_to_snake_case("SCREAM"), eq
        "scream"
    );
}
