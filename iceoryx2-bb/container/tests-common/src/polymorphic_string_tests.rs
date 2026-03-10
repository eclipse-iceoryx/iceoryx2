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

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_bb_container::string::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_testing::assert_that;

const SUT_CAPACITY: usize = 256;

struct Test {
    raw_memory: UnsafeCell<Box<[u8; core::mem::size_of::<u8>() * (SUT_CAPACITY * 3)]>>,
    allocator: UnsafeCell<Option<Box<BumpAllocator>>>,
}

impl Test {
    fn new() -> Self {
        Self {
            raw_memory: UnsafeCell::new(Box::new(
                [0u8; core::mem::size_of::<u8>() * (SUT_CAPACITY * 3)],
            )),
            allocator: UnsafeCell::new(None),
        }
    }

    fn allocator<'a>(&'a self) -> &'a BumpAllocator {
        unsafe {
            if (*self.allocator.get()).is_none() {
                *self.allocator.get() = Some(Box::new(BumpAllocator::new(
                    (*self.raw_memory.get()).as_mut_ptr(),
                )))
            }
        };

        unsafe { (*self.allocator.get()).as_ref().unwrap() }
    }

    fn create_sut<'a>(
        &'a self,
        capacity: usize,
    ) -> Result<PolymorphicString<'a, BumpAllocator>, AllocationError> {
        PolymorphicString::new(self.allocator(), capacity)
    }
}

#[test]
fn try_clone_clones_empty_string() {
    let test = Test::new();
    let sut = test.create_sut(3).unwrap();
    let sut_clone = sut.try_clone().unwrap();

    assert_that!(sut, eq sut_clone);
    assert_that!(sut.len(), eq 0);
    assert_that!(sut_clone.len(), eq 0);
    assert_that!(sut.capacity(), eq 3);
    assert_that!(sut_clone.capacity(), eq 3);
}

#[test]
fn try_clone_clones_filled_string() {
    let test = Test::new();
    let mut sut = test.create_sut(99).unwrap();
    assert_that!(sut.push_bytes(b"all glory to hypnofrog!"), is_ok);
    let sut_clone = sut.try_clone().unwrap();

    assert_that!(sut.len(), eq 23);
    assert_that!(sut_clone.len(), eq 23);
    assert_that!(sut.as_bytes(), eq b"all glory to hypnofrog!");
    assert_that!(sut_clone.as_bytes(), eq b"all glory to hypnofrog!");
}
