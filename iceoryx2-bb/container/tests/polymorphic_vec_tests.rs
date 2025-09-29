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

use std::cell::UnsafeCell;

use iceoryx2_bb_container::polymorphic_vec::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_testing::{assert_that, lifetime_tracker::LifetimeTracker};

const SUT_CAPACITY: usize = 10;

struct Test {
    raw_memory: UnsafeCell<Box<[u8; core::mem::size_of::<LifetimeTracker>() * (SUT_CAPACITY * 3)]>>,
    allocator: UnsafeCell<Option<Box<BumpAllocator>>>,
}

impl Test {
    fn new() -> Self {
        Self {
            raw_memory: UnsafeCell::new(Box::new(
                [0u8; core::mem::size_of::<LifetimeTracker>() * (SUT_CAPACITY * 3)],
            )),
            allocator: UnsafeCell::new(None),
        }
    }

    fn create_sut<'a>(
        &'a self,
        capacity: usize,
    ) -> Result<PolymorphicVec<'a, LifetimeTracker, BumpAllocator>, AllocationError> {
        unsafe {
            if (*self.allocator.get()).is_none() {
                *self.allocator.get() = Some(Box::new(BumpAllocator::new(
                    (*self.raw_memory.get()).as_mut_ptr(),
                )))
            }
        };

        PolymorphicVec::new(
            unsafe { (*self.allocator.get()).as_ref().unwrap() },
            capacity,
        )
    }
}

#[test]
fn try_clone_clones_empty_vec() {
    let test = Test::new();
    let sut = test.create_sut(3).unwrap();
    let sut_clone = sut.try_clone().unwrap();

    assert_that!(sut, eq sut_clone);
    assert_that!(sut.len(), eq 0);
    assert_that!(sut_clone.len(), eq 0);
}

#[test]
fn try_clone_filled_empty_vec() {
    let test = Test::new();
    let mut sut = test.create_sut(3).unwrap();

    for n in 0..3 {
        sut.push(LifetimeTracker::new_with_value(n + 2));
    }

    let sut_clone = sut.try_clone().unwrap();

    assert_that!(sut, eq sut_clone);
    assert_that!(sut.len(), eq 3);
    assert_that!(sut_clone.len(), eq 3);
}
