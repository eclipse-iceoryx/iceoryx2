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

use alloc::boxed::Box;
use core::{alloc::Layout, ptr::NonNull};

use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_cal::shm_allocator::{
    AllocationStrategy, PointerOffset, ShmAllocator, shm_bump_allocator::*,
};

const MAX_SUPPORTED_ALIGNMENT: usize = 4096;
const MEM_SIZE: usize = 16384 * 10;
const PAYLOAD_SIZE: usize = 8192;

struct Test {
    _payload_memory: Box<[u8; MEM_SIZE]>,
    base_address: NonNull<[u8]>,
    sut: Box<BumpAllocator>,
}

impl Test {
    fn new() -> Self {
        let mut payload_memory = Box::new([0u8; MEM_SIZE]);
        let base_address =
            unsafe { NonNull::<[u8]>::new_unchecked(&mut payload_memory[0..PAYLOAD_SIZE]) };
        let allocator = iceoryx2_bb_memory::bump_allocator::BumpAllocator::new(
            unsafe { NonNull::new_unchecked(payload_memory[PAYLOAD_SIZE..].as_mut_ptr()) },
            MEM_SIZE,
        );
        let config = Config::default();
        let mut sut = Box::new(unsafe {
            BumpAllocator::new_uninit(MAX_SUPPORTED_ALIGNMENT, base_address, &config)
        });

        unsafe { sut.init(&allocator).unwrap() };

        Self {
            _payload_memory: payload_memory,
            base_address,
            sut,
        }
    }

    fn generate_layout(size: usize) -> Layout {
        unsafe { Layout::from_size_align_unchecked(size, 1) }
    }

    fn offset_to_ptr(&mut self, offset: PointerOffset) -> *mut u8 {
        unsafe { self.base_address.as_mut().as_mut_ptr().add(offset.offset()) }
    }
}

#[test]
fn initial_setup_hint_is_layout_times_number_of_chunks() {
    let layout = Layout::from_size_align(64, 2).unwrap();
    let max_number_of_chunks = 54;
    let hint = BumpAllocator::initial_setup_hint(layout, max_number_of_chunks);

    assert_that!(hint.payload_size, eq layout.size() * max_number_of_chunks);
}

fn no_new_resize_hint_when_there_is_memory_available(strategy: AllocationStrategy) {
    let test_context = Test::new();
    let hint = test_context
        .sut
        .resize_hint(Layout::from_size_align(8, 2).unwrap(), strategy);

    assert_that!(hint.payload_size, eq test_context.sut.total_space());
}

#[test]
fn no_new_resize_hint_with_power_of_two_when_there_is_memory_available() {
    no_new_resize_hint_when_there_is_memory_available(AllocationStrategy::PowerOfTwo)
}

#[test]
fn no_new_resize_hint_with_best_fit_when_there_is_memory_available() {
    no_new_resize_hint_when_there_is_memory_available(AllocationStrategy::BestFit)
}

#[test]
fn new_resize_hint_with_power_of_two_when_there_is_not_enough_memory_available() {
    let test_context = Test::new();
    let layout = Layout::from_size_align(test_context.sut.total_space() + 1, 1).unwrap();
    let hint = test_context
        .sut
        .resize_hint(layout, AllocationStrategy::PowerOfTwo);
    assert_that!(
        hint.payload_size,
        eq(test_context.sut.total_space() + layout.size()).next_power_of_two()
    );
}

#[test]
fn new_resize_hint_with_best_fit_when_there_is_not_enough_memory_available() {
    let test_context = Test::new();
    let layout = Layout::from_size_align(test_context.sut.total_space() + 1, 1).unwrap();
    let hint = test_context
        .sut
        .resize_hint(layout, AllocationStrategy::BestFit);
    assert_that!(
        hint.payload_size,
        eq(test_context.sut.total_space() + layout.size())
    );
}

#[test]
fn growing_last_chunk_and_keep_content_at_front_works() {
    let mut test = Test::new();
    let old_layout = Test::generate_layout(4);
    let offset = unsafe { test.sut.allocate(old_layout) }.unwrap();
    let ptr = test.offset_to_ptr(offset);

    for n in 0..4 {
        unsafe { *ptr.add(n) = n as u8 };
    }

    let new_layout = Test::generate_layout(9);
    let offset = unsafe {
        test.sut
            .grow(
                offset,
                old_layout,
                new_layout,
                iceoryx2_cal::shm_allocator::ContentPlacement::Front,
            )
            .unwrap()
    };
    let ptr = test.offset_to_ptr(offset);

    for n in 0..4 {
        assert_that!(unsafe { *ptr.add(n) }, eq n as u8);
    }
}

#[test]
fn growing_last_chunk_and_keep_content_at_back_works() {
    let mut test = Test::new();
    let old_layout = Test::generate_layout(4);
    let offset = unsafe { test.sut.allocate(old_layout) }.unwrap();
    let ptr = test.offset_to_ptr(offset);

    for n in 0..4 {
        unsafe { *ptr.add(n) = n as u8 + 5u8 };
    }

    let new_layout = Test::generate_layout(9);
    let offset = unsafe {
        test.sut
            .grow(
                offset,
                old_layout,
                new_layout,
                iceoryx2_cal::shm_allocator::ContentPlacement::Back,
            )
            .unwrap()
    };
    let ptr = test.offset_to_ptr(offset);

    for n in 5..9 {
        assert_that!(unsafe { *ptr.add(n) }, eq n as u8);
    }
}

#[test]
fn growing_middle_chunk_and_keep_content_at_front_works() {
    let mut test = Test::new();
    let old_layout = Test::generate_layout(6);
    let offset = unsafe { test.sut.allocate(old_layout) }.unwrap();
    let ptr = test.offset_to_ptr(offset);

    for n in 0..6 {
        unsafe { *ptr.add(n) = n as u8 * 2u8 };
    }

    let middle_chunk = unsafe { test.sut.allocate(Test::generate_layout(7)).unwrap() };
    let ptr = test.offset_to_ptr(middle_chunk);

    for n in 0..7 {
        unsafe { *ptr.add(n) = 123u8 };
    }

    let new_layout = Test::generate_layout(10);
    let offset = unsafe {
        test.sut
            .grow(
                offset,
                old_layout,
                new_layout,
                iceoryx2_cal::shm_allocator::ContentPlacement::Front,
            )
            .unwrap()
    };
    let ptr = test.offset_to_ptr(offset);

    for n in 0..6 {
        assert_that!(unsafe { *ptr.add(n) }, eq n as u8 * 2u8);
    }
    for n in 7..10 {
        unsafe { *ptr.add(n) = 41u8 };
    }

    let ptr = test.offset_to_ptr(middle_chunk);

    for n in 0..7 {
        assert_that!(unsafe { *ptr.add(n) }, eq 123u8);
    }
}

#[test]
fn growing_middle_chunk_and_keep_content_at_front_works() {
    let mut test = Test::new();
    let old_layout = Test::generate_layout(6);
    let offset = unsafe { test.sut.allocate(old_layout) }.unwrap();
    let ptr = test.offset_to_ptr(offset);

    for n in 0..6 {
        unsafe { *ptr.add(n) = n as u8 * 2u8 };
    }

    let middle_chunk = unsafe { test.sut.allocate(Test::generate_layout(7)).unwrap() };
    let ptr = test.offset_to_ptr(middle_chunk);

    for n in 0..7 {
        unsafe { *ptr.add(n) = 123u8 };
    }

    let new_layout = Test::generate_layout(10);
    let offset = unsafe {
        test.sut
            .grow(
                offset,
                old_layout,
                new_layout,
                iceoryx2_cal::shm_allocator::ContentPlacement::Front,
            )
            .unwrap()
    };
    let ptr = test.offset_to_ptr(offset);

    for n in 0..6 {
        assert_that!(unsafe { *ptr.add(n) }, eq n as u8 * 2u8);
    }
    for n in 7..10 {
        unsafe { *ptr.add(n) = 41u8 };
    }

    let ptr = test.offset_to_ptr(middle_chunk);

    for n in 0..7 {
        assert_that!(unsafe { *ptr.add(n) }, eq 123u8);
    }
}
