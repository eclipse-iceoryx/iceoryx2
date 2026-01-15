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

use iceoryx2_bb_container::vector::relocatable_vec::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_testing::assert_that;

#[test]
#[should_panic]
fn double_init_call_causes_panic() {
    const CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<u128>::const_memory_size(CAPACITY);
    let mut memory = [0u8; MEM_SIZE];
    let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());
    let mut sut = unsafe { RelocatableVec::<u128>::new_uninit(CAPACITY) };
    unsafe { sut.init(&bump_allocator).expect("sut init failed") };

    unsafe { sut.init(&bump_allocator).expect("sut init failed") };
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn panic_is_called_in_debug_mode_if_vec_is_not_initialized() {
    const CAPACITY: usize = 12;
    let mut sut = unsafe { RelocatableVec::<u8>::new_uninit(CAPACITY) };
    assert_that!(sut.push(0), is_ok);
}

#[test]
fn two_vectors_with_same_content_are_equal() {
    const SUT_CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<usize>::const_memory_size(SUT_CAPACITY);
    let mut memory_1 = [0u8; MEM_SIZE];
    let mut memory_2 = [0u8; MEM_SIZE];
    let bump_allocator_1 = BumpAllocator::new(memory_1.as_mut_ptr());
    let bump_allocator_2 = BumpAllocator::new(memory_2.as_mut_ptr());
    let mut sut_1 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_1.init(&bump_allocator_1).unwrap() };
    let mut sut_2 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_2.init(&bump_allocator_2).unwrap() };

    for n in 0..SUT_CAPACITY {
        assert_that!(sut_1.push(4 * n + 3), is_ok);
        assert_that!(sut_2.insert(n, 4 * n + 3), is_ok);
    }

    assert_that!(sut_1, eq sut_2);
}

#[test]
fn two_vectors_with_different_content_are_not_equal() {
    const SUT_CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<usize>::const_memory_size(SUT_CAPACITY);
    let mut memory_1 = [0u8; MEM_SIZE];
    let mut memory_2 = [0u8; MEM_SIZE];
    let bump_allocator_1 = BumpAllocator::new(memory_1.as_mut_ptr());
    let bump_allocator_2 = BumpAllocator::new(memory_2.as_mut_ptr());
    let mut sut_1 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_1.init(&bump_allocator_1).unwrap() };
    let mut sut_2 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_2.init(&bump_allocator_2).unwrap() };

    for n in 0..SUT_CAPACITY {
        assert_that!(sut_1.push(4 * n + 3), is_ok);
        assert_that!(sut_2.insert(n, 4 * n + 3), is_ok);
    }

    sut_2[5] = 0;

    assert_that!(sut_1, ne sut_2);
}

#[test]
fn two_vectors_with_different_len_are_not_equal() {
    const SUT_CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<usize>::const_memory_size(SUT_CAPACITY);
    let mut memory_1 = [0u8; MEM_SIZE];
    let mut memory_2 = [0u8; MEM_SIZE];
    let bump_allocator_1 = BumpAllocator::new(memory_1.as_mut_ptr());
    let bump_allocator_2 = BumpAllocator::new(memory_2.as_mut_ptr());
    let mut sut_1 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_1.init(&bump_allocator_1).unwrap() };
    let mut sut_2 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_2.init(&bump_allocator_2).unwrap() };

    for n in 0..SUT_CAPACITY {
        assert_that!(sut_1.push(4 * n + 3), is_ok);
        assert_that!(sut_2.insert(n, 4 * n + 3), is_ok);
    }

    sut_2.pop();

    assert_that!(sut_1, ne sut_2);
}
