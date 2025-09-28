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

use iceoryx2_bb_container::static_vec::StaticVec;
use iceoryx2_bb_testing::{assert_that, lifetime_tracker::LifetimeTracker};

const SUT_CAPACITY: usize = 10;

#[test]
fn new_created_vec_is_empty() {
    let sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    assert_that!(sut.is_empty(), eq true);
    assert_that!(sut.len(), eq 0);
    assert_that!(sut.is_full(), eq false);
}

#[test]
fn default_created_vec_is_empty() {
    let sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::default();

    assert_that!(sut.is_empty(), eq true);
    assert_that!(sut.len(), eq 0);
    assert_that!(sut.is_full(), eq false);
}

#[test]
fn push_adds_element_at_the_end() {
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    for number in 0..SUT_CAPACITY {
        assert_that!(sut.push(LifetimeTracker::new_with_value(number)), eq true);

        for idx in 0..number + 1 {
            assert_that!(sut[idx].value, eq idx);
        }
    }
}

#[test]
fn push_more_elements_than_capacity_fails() {
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    for number in 0..SUT_CAPACITY {
        assert_that!(sut.push(LifetimeTracker::new_with_value(2 * number)), eq true);
    }

    for number in 0..SUT_CAPACITY {
        assert_that!(sut.push(LifetimeTracker::new_with_value(number)), eq false);

        for idx in 0..SUT_CAPACITY {
            assert_that!(sut[idx].value, eq 2 * idx);
        }
    }
}

#[test]
fn pop_returns_none_on_empty_when_empty() {
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    assert_that!(sut.pop(), is_none);
}

#[test]
fn pop_removes_last_element() {
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    for number in 0..SUT_CAPACITY {
        assert_that!(sut.push(LifetimeTracker::new_with_value(4 * number + 1)), eq true);
    }

    for number in (0..SUT_CAPACITY).rev() {
        let value = sut.pop();
        assert_that!(value, is_some);
        assert_that!(value.unwrap().value, eq 4 * number + 1);
    }

    assert_that!(sut.pop(), is_none);
}

#[test]
fn truncate_does_nothing_when_new_len_is_larger_than_current_len() {
    let half_capacity = SUT_CAPACITY / 2;
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    for number in 0..half_capacity {
        assert_that!(sut.push(LifetimeTracker::new_with_value(4 * number + 3)), eq true);
    }

    sut.truncate(SUT_CAPACITY);

    for idx in 0..half_capacity {
        assert_that!(sut[idx].value, eq 4 * idx + 3);
    }
}

#[test]
fn truncate_drops_all_elements_right_of_new_len() {
    let half_capacity = SUT_CAPACITY / 2;
    let tracker = LifetimeTracker::start_tracking();
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    for number in 0..SUT_CAPACITY {
        assert_that!(sut.push(LifetimeTracker::new_with_value(5 * number + 7)), eq true);
    }
    assert_that!(tracker.number_of_living_instances(), eq SUT_CAPACITY);

    sut.truncate(half_capacity);
    assert_that!(sut.len(), eq half_capacity);
    assert_that!(tracker.number_of_living_instances(), eq half_capacity);

    for idx in 0..half_capacity {
        assert_that!(sut[idx].value, eq 5 * idx + 7);
    }
}

#[test]
fn truncate_drops_elements_in_reverse_order() {
    let half_capacity = SUT_CAPACITY / 2;
    let tracker = LifetimeTracker::start_tracking();
    let mut sut = StaticVec::<LifetimeTracker, SUT_CAPACITY>::new();

    for number in 0..SUT_CAPACITY {
        assert_that!(sut.push(LifetimeTracker::new_with_value(number)), eq true);
    }

    sut.truncate(half_capacity);

    for (n, drop_value) in tracker.drop_order().iter().enumerate() {
        assert_that!(*drop_value, eq SUT_CAPACITY - n - 1);
    }
}
