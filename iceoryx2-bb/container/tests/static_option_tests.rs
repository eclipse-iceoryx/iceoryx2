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

use iceoryx2_bb_container::static_option::StaticOption;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_testing::{assert_that, lifetime_tracker::LifetimeTracker};
use serde_test::{assert_tokens, Token};

#[test]
fn default_created_option_is_empty() {
    let sut = StaticOption::<i32>::default();

    assert_that!(sut.is_some(), eq false);
    assert_that!(sut.is_none(), eq true);
}

#[test]
fn when_empty_as_option_returns_empty_option() {
    let sut = StaticOption::<i32>::default();

    assert_that!(sut.as_option(), eq None);
}

#[test]
fn when_with_value_as_option_returns_option_with_reference_to_that_value() {
    let sut = StaticOption::<i32>::some(1234);

    assert_that!(*sut.as_option().unwrap(), eq 1234);
}

#[test]
fn when_empty_as_option_mut_returns_empty_option() {
    let mut sut = StaticOption::<i32>::default();

    assert_that!(sut.as_option_mut(), eq None);
}

#[test]
fn when_with_value_as_option_returns_option_with_mut_reference_to_that_value() {
    let mut sut = StaticOption::<i32>::some(4313);

    assert_that!(*sut.as_option_mut().unwrap(), eq 4313);
}

#[test]
fn when_empty_as_deref_returns_empty_option() {
    let sut = StaticOption::<Vec<i32>>::default();

    assert_that!(sut.as_deref().is_none(), eq true);
}

#[test]
fn when_with_value_as_deref_returns_ref_to_target() {
    let sut = StaticOption::<Vec<i32>>::some(vec![1, 2, 3]);

    assert_that!(sut.as_deref().unwrap(), eq [1,2,3]);
}

#[test]
fn when_empty_as_deref_mut_returns_empty_option() {
    let mut sut = StaticOption::<Vec<i32>>::default();

    assert_that!(sut.as_deref_mut().is_none(), eq true);
}

#[test]
fn when_with_value_as_deref_mut_returns_ref_to_target() {
    let mut sut = StaticOption::<Vec<i32>>::some(vec![6, 6, 3]);

    assert_that!(sut.as_deref_mut().unwrap(), eq [6,6,3]);
}

#[test]
fn when_empty_as_ref_returns_empty_option() {
    let sut = StaticOption::<i32>::default();

    assert_that!(sut.as_ref().is_none(), eq true);
}

#[test]
fn when_with_value_as_ref_returns_ref_to_target() {
    let sut = StaticOption::<i32>::some(98123);

    assert_that!(*sut.as_ref().unwrap(), eq 98123);
}

#[test]
fn when_empty_as_mut_returns_empty_option() {
    let mut sut = StaticOption::<i32>::default();

    assert_that!(sut.as_mut().is_none(), eq true);
}

#[test]
fn when_with_value_as_mut_returns_ref_to_target() {
    let mut sut = StaticOption::<i32>::some(553);

    assert_that!(*sut.as_mut().unwrap(), eq 553);
}

#[test]
fn expect_returns_value() {
    let sut = StaticOption::<i32>::some(1553);

    assert_that!(sut.expect(""), eq 1553);
}

#[should_panic]
#[test]
fn expect_panics_when_empty() {
    let sut = StaticOption::<i32>::none();

    sut.expect("");
}

#[test]
fn none_creates_empty_value() {
    let sut = StaticOption::<i32>::none();

    assert_that!(sut.is_none(), eq true);
    assert_that!(sut.is_some(), eq false);
}

#[test]
fn some_creates_option_that_contains_the_value() {
    let sut = StaticOption::<i32>::some(89928);

    assert_that!(sut.is_none(), eq false);
    assert_that!(sut.is_some(), eq true);
    assert_that!(sut.unwrap(), eq 89928);
}

#[test]
fn inspect_callback_is_not_called_when_empty() {
    let sut = StaticOption::<i32>::none();
    let mut callback_was_called = false;
    sut.inspect(|_| callback_was_called = true);

    assert_that!(callback_was_called, eq false);
}

#[test]
fn inspect_callback_is_called_when_it_contains_a_value() {
    let sut = StaticOption::<i32>::some(778);
    let mut callback_was_called = false;
    sut.inspect(|v| {
        callback_was_called = true;
        assert_that!(*v, eq 778);
    });

    assert_that!(callback_was_called, eq true);
}
