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

extern crate alloc;

use std::alloc::Layout;

use iceoryx2_bb_elementary_traits::pointer::Pointer;
use iceoryx2_bb_flatbuffers::{
    AllocationStrategy, Allocator, ResizableMemoryBuilder, ResizableMemoryError,
};
use iceoryx2_bb_memory::{heap_allocator::*, pool_allocator::AllocationGrowError};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn reserved_header_len_is_accounted_for_in_len() {
    const HEADER_SIZE: usize = 3;
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::new::<u64>();
    let memory = heap_allocator.allocate(initial_layout).unwrap();
    let sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .reserved_header_len(HEADER_SIZE)
        .allocation_strategy(AllocationStrategy::Static)
        .create(heap_allocator)
        .unwrap();

    assert_that!(sut.len(), eq initial_layout.size() - HEADER_SIZE);
}

#[test]
pub fn reserved_header_cannot_be_accessed() {
    const HEADER_SIZE: usize = 3;
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::new::<u64>();
    let mut memory = heap_allocator.allocate(initial_layout).unwrap();

    for n in 0..HEADER_SIZE {
        unsafe { *memory.as_mut_ptr().add(n) = 53 };
    }

    let mut sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .reserved_header_len(HEADER_SIZE)
        .allocation_strategy(AllocationStrategy::Static)
        .create(heap_allocator)
        .unwrap();

    for content in &mut *sut {
        *content = 79;
    }

    for n in 0..HEADER_SIZE {
        assert_that!(unsafe {*memory.as_mut_ptr().add(n) }, eq 53);
    }
}

#[test]
pub fn reserved_header_cannot_larger_than_initial_size() {
    const HEADER_SIZE: usize = 9;
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::new::<u64>();
    let memory = heap_allocator.allocate(initial_layout).unwrap();

    let sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .reserved_header_len(HEADER_SIZE)
        .allocation_strategy(AllocationStrategy::Static)
        .create(heap_allocator);

    assert_that!(sut.err(), eq Some(ResizableMemoryError::ReservedHeaderLenExceedsInitialSize));
}

#[test]
pub fn allocation_strategy_static_does_now_allow_growing() {
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::new::<u64>();
    let memory = heap_allocator.allocate(initial_layout).unwrap();

    let mut sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .allocation_strategy(AllocationStrategy::Static)
        .create(heap_allocator)
        .unwrap();

    let result = sut.grow_downwards();
    assert_that!(result.err(), eq Some(AllocationGrowError::OutOfMemory));
}

#[test]
pub fn allocation_strategy_best_fit_grows_with_min_alignment() {
    const ALIGN: usize = 128;
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::from_size_align(ALIGN * 2, ALIGN).unwrap();
    let memory = heap_allocator.allocate(initial_layout).unwrap();

    let mut sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .allocation_strategy(AllocationStrategy::BestFit)
        .create(heap_allocator)
        .unwrap();

    for _ in 0..10 {
        let current_size = sut.len();
        sut.grow_downwards().unwrap();
        let new_size = sut.len();

        assert_that!(new_size, ge current_size + ALIGN);
    }
}

#[test]
pub fn allocation_strategy_power_of_two_grows_exponentially() {
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::from_size_align(5, 1).unwrap();
    let memory = heap_allocator.allocate(initial_layout).unwrap();

    let mut sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create(heap_allocator)
        .unwrap();

    for _ in 0..10 {
        let current_size = sut.len();
        sut.grow_downwards().unwrap();
        let new_size = sut.len();

        assert_that!(new_size, ge(current_size + 1).next_power_of_two());
    }
}

#[test]
pub fn growing_keeps_header_in_front() {
    const HEADER_SIZE: usize = 7;
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::new::<u64>();
    let mut memory = heap_allocator.allocate(initial_layout).unwrap();

    for n in 0..HEADER_SIZE {
        unsafe { *memory.as_mut_ptr().add(n) = 59 };
    }

    let mut sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .reserved_header_len(HEADER_SIZE)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create(heap_allocator)
        .unwrap();

    for _ in 0..10 {
        sut.grow_downwards().unwrap();
        let header_ptr = unsafe { sut.as_ptr().offset(-(HEADER_SIZE as isize)) };
        for n in 0..HEADER_SIZE {
            assert_that!(unsafe { *header_ptr.add(n) }, eq 59);
        }
    }
}

#[test]
pub fn growing_keeps_content_at_end() {
    const HEADER_SIZE: usize = 7;
    let heap_allocator = HeapAllocator::new();
    let initial_layout = Layout::new::<u64>();
    let mut memory = heap_allocator.allocate(initial_layout).unwrap();

    for n in 0..HEADER_SIZE {
        unsafe { *memory.as_mut_ptr().add(n) = 67 };
    }

    let mut sut = ResizableMemoryBuilder::new(memory)
        .initial_layout(initial_layout)
        .reserved_header_len(HEADER_SIZE)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create(heap_allocator)
        .unwrap();

    for n in 0..10 {
        for element in &mut *sut {
            *element = n + 1;
        }

        let previous_len = sut.len();
        sut.grow_downwards().unwrap();

        for element in &sut[sut.len() - previous_len..sut.len()] {
            assert_that!(*element, eq n + 1);
        }
    }
}
