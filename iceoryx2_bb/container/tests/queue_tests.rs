// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::queue::*;
use iceoryx2_bb_memory::{bump_allocator::BumpAllocator, memory::Memory};
use iceoryx2_bb_testing::assert_that;
use pin_init::init_stack;

const SUT_CAPACITY: usize = 128;
type Sut = FixedSizeQueue<usize, SUT_CAPACITY>;

#[test]
fn queue_push_pop_works_with_uninitialized_memory() {
    init_stack!(
        memory =
            Memory::<{ Queue::<usize>::const_memory_size(129_usize) }, BumpAllocator>::new_filled(
                0xff,
            )
    );
    let memory = memory.unwrap();
    let mut sut = unsafe { RelocatableQueue::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { assert_that!(sut.init(memory.allocator()), is_ok) };

    for i in 0..sut.capacity() {
        let element = i * 2 + 3;
        assert_that!(sut.is_full(), eq false);
        assert_that!(unsafe { sut.push(element) }, eq true);
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len i + 1);
    }
    assert_that!(sut.is_full(), eq true);

    for i in 0..sut.capacity() {
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len sut.capacity() - i);
        let result = unsafe { sut.pop() };
        assert_that!(sut.is_full(), eq false);
        assert_that!(result, eq Some(i * 2 + 3));
    }

    assert_that!(sut, is_empty);
    assert_that!(sut, len 0);
}

#[test]
fn fixed_size_queue_capacity_is_correct() {
    let sut = Sut::new();
    assert_that!(sut.capacity(), eq SUT_CAPACITY);
}

#[test]
fn fixed_size_queue_newly_created_buffer_is_empty() {
    let mut sut = Sut::new();
    assert_that!(sut, is_empty);
    assert_that!(sut, len 0);
    assert_that!(sut.pop(), is_none);
    assert_that!(sut.is_full(), eq false);
}

#[test]
fn fixed_size_queue_push_pop_works() {
    let mut sut = Sut::new();

    for i in 0..sut.capacity() {
        let element = i * 2 + 3;
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut.push(element), eq true);
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len i + 1);
    }
    assert_that!(sut.is_full(), eq true);

    for i in 0..sut.capacity() {
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len sut.capacity() - i);
        let result = sut.pop();
        assert_that!(sut.is_full(), eq false);
        assert_that!(result, eq Some(i * 2 + 3));
    }

    assert_that!(sut, is_empty);
    assert_that!(sut, len 0);
}

#[test]
fn fixed_size_queue_valid_after_move() {
    let mut sut = Sut::new();

    for i in 0..sut.capacity() {
        let element = i * 2 + 3;
        assert_that!(sut.push(element), eq true);
    }

    let mut sut2 = sut;

    for i in 0..sut2.capacity() {
        let result = sut2.pop();
        assert_that!(result, eq Some(i * 2 + 3));
    }
}

#[test]
fn fixed_size_queue_push_pop_alteration_works() {
    let mut sut = Sut::new();

    let mut push_counter: usize = 0;
    let mut pop_counter: usize = 0;
    for _ in 0..sut.capacity() / 3 {
        for _ in 0..5 {
            let element = push_counter * 4 + 1;
            push_counter += 1;
            assert_that!(sut.push(element), eq true);
        }

        for _ in 0..3 {
            let result = sut.pop();
            assert_that!(result, eq Some(pop_counter * 4 + 1));
            pop_counter += 1;
        }
    }
}

#[test]
fn fixed_size_queue_clear_works() {
    let mut sut = Sut::new();

    for i in 0..sut.capacity() {
        assert_that!(sut.push(i), eq true);
    }

    sut.clear();
    assert_that!(sut, is_empty);
    assert_that!(sut, len 0);
    assert_that!(sut.pop(), is_none);
}

#[test]
fn fixed_size_queue_overflow_works() {
    let mut sut = Sut::new();

    for i in 0..sut.capacity() {
        let element = i;
        assert_that!(sut.push_with_overflow(element), is_none);
    }

    for i in 0..sut.capacity() {
        let element = (i + 5) * sut.capacity();
        let result = sut.push_with_overflow(element);
        assert_that!(result, eq Some(i));
    }

    for i in 0..sut.capacity() {
        let element = (i + 5) * sut.capacity();
        let result = sut.pop();
        assert_that!(result, eq Some(element));
    }
}

#[test]
fn fixed_size_queue_iterate_with_get() {
    let mut sut = Sut::new();

    for i in 0..sut.capacity() / 2 {
        sut.push_with_overflow(i);
    }

    for i in 0..sut.capacity() {
        sut.push_with_overflow(2 * i + 25);
    }

    for i in 0..sut.len() {
        assert_that!(sut.get(i), eq 2 * i + 25);
    }
}
