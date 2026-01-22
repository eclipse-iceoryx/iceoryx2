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

use std::mem::MaybeUninit;

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

#[test]
fn replace_returns_none_when_option_is_empty() {
    let mut sut = StaticOption::<i32>::none();

    assert_that!(sut.replace(89123).is_none(), eq true);
    assert_that!(sut.unwrap(), eq 89123);
}

#[test]
fn replace_returns_value_when_option_contains_value() {
    let mut sut = StaticOption::<i32>::some(9012);

    assert_that!(sut.replace(891231), eq StaticOption::some(9012));
    assert_that!(sut.unwrap(), eq 891231);
}

#[test]
fn take_empties_sut_and_returns_content() {
    let mut sut_full = StaticOption::<i32>::some(90125);
    let mut sut_empty = StaticOption::<i32>::none();

    assert_that!(sut_full.take(), eq StaticOption::some(90125));
    assert_that!(sut_empty.take(), eq StaticOption::none());

    assert_that!(sut_full.is_none(), eq true);
    assert_that!(sut_empty.is_none(), eq true);
}

#[test]
fn take_if_does_not_call_callback_when_empty() {
    let mut sut = StaticOption::<i32>::none();
    let mut callback_was_called = false;
    let ret_val = sut.take_if(|_| {
        callback_was_called = true;
        false
    });

    assert_that!(callback_was_called, eq false);
    assert_that!(ret_val, eq StaticOption::none());
    assert_that!(sut, eq StaticOption::none());
}

#[test]
fn take_if_returns_none_when_callback_returns_false() {
    let mut sut = StaticOption::<i32>::some(551);
    let mut callback_was_called = false;
    let ret_val = sut.take_if(|v| {
        assert_that!(*v, eq 551);
        callback_was_called = true;
        false
    });

    assert_that!(callback_was_called, eq true);
    assert_that!(ret_val, eq StaticOption::none());
    assert_that!(sut, eq StaticOption::some(551));
}

#[test]
fn take_if_returns_value_and_empties_option_when_callback_returns_true() {
    let mut sut = StaticOption::<i32>::some(1551);
    let mut callback_was_called = false;
    let ret_val = sut.take_if(|v| {
        assert_that!(*v, eq 1551);
        callback_was_called = true;
        true
    });

    assert_that!(callback_was_called, eq true);
    assert_that!(ret_val, eq StaticOption::some(1551));
    assert_that!(sut, eq StaticOption::none());
}

#[test]
fn unwrap_returns_value_when_it_has_one() {
    let sut = StaticOption::<i32>::some(15511);

    assert_that!(sut.unwrap(), eq 15511);
}

#[should_panic]
#[test]
fn unwrap_panics_when_empty() {
    let sut = StaticOption::<i32>::none();

    sut.unwrap();
}

#[test]
fn unwrap_or_returns_provided_value_when_empty() {
    let sut = StaticOption::<i32>::none();

    assert_that!(sut.unwrap_or(8192), eq 8192);
}

#[test]
fn unwrap_or_returns_value() {
    let sut = StaticOption::<i32>::some(661);

    assert_that!(sut.unwrap_or(8), eq 661);
}

#[test]
fn unwrap_or_default_returns_default_when_empty() {
    let sut = StaticOption::<i32>::none();

    assert_that!(sut.unwrap_or_default(), eq i32::default());
}

#[test]
fn unwrap_or_default_returns_value() {
    let sut = StaticOption::<i32>::some(981);

    assert_that!(sut.unwrap_or_default(), eq 981);
}

#[test]
fn unwrap_or_else_returns_callable_value_when_empty() {
    let sut = StaticOption::<i32>::none();

    assert_that!(sut.unwrap_or_else(|| 8127), eq 8127);
}

#[test]
fn unwrap_or_else_returns_value() {
    let sut = StaticOption::<i32>::some(113);

    let mut callable_was_called = false;
    assert_that!(sut.unwrap_or_else(|| {callable_was_called = true; 8127}), eq 113);
    assert_that!(callable_was_called, eq false);
}

#[test]
fn unwrap_unchecked_returns_value() {
    let sut = StaticOption::<i32>::some(1113);

    assert_that!(unsafe { sut.unwrap_unchecked() }, eq 1113);
}

#[test]
fn element_is_dropped_on_option_drop() {
    let tracker = LifetimeTracker::start_tracking();
    let sut = StaticOption::<LifetimeTracker>::some(LifetimeTracker::new());
    assert_that!(tracker.number_of_living_instances(), eq 1);

    drop(sut);
    assert_that!(tracker.number_of_living_instances(), eq 0);
}

#[test]
fn debug_fmt_works() {
    let sut_none = StaticOption::<i32>::none();
    let sut_some = StaticOption::<i32>::some(112);

    assert_that!(format!("{:?}", sut_none), eq "StaticOption<i32>::none()");
    assert_that!(format!("{:?}", sut_some), eq "StaticOption<i32>::some(112)");
}

#[test]
fn clone_works() {
    let sut_orig_some = StaticOption::<i32>::some(8812);
    let sut_orig_none = StaticOption::<i32>::none();
    let sut_clone_some = sut_orig_some.clone();
    let sut_clone_none = sut_orig_none.clone();

    assert_that!(sut_orig_some, eq sut_clone_some);
    assert_that!(sut_orig_none, eq sut_clone_none);
    assert_that!(sut_clone_none, ne sut_clone_some);
}

#[test]
fn placement_default_works() {
    let mut raw_sut = MaybeUninit::<StaticOption<i32>>::uninit();
    unsafe { StaticOption::placement_default(raw_sut.as_mut_ptr()) };

    assert_that!(unsafe { raw_sut.assume_init().is_none() }, eq true);
}

#[test]
fn serialization_works() {
    let sut_none = StaticOption::<i32>::none();
    let sut_some = StaticOption::<i32>::some(551);

    assert_tokens(&sut_none, &[Token::None]);
    assert_tokens(&sut_some, &[Token::Some, Token::I32(551)]);
}
