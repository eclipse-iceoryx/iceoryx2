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

use iceoryx2_bb_elementary_traits::allocator::*;
use iceoryx2_bb_memory::bump_allocator::*;
use iceoryx2_bb_testing::assert_that;

struct TestFixture {
    raw_memory: [u8; TestFixture::memory_size()],
}

impl TestFixture {
    fn new() -> Self {
        let mut test = TestFixture {
            raw_memory: [0; TestFixture::memory_size()],
        };

        for i in 0..TestFixture::memory_size() {
            test.raw_memory[i] = 255;
        }

        test
    }

    const fn memory_size() -> usize {
        8192
    }

    fn create_bump_allocator(&mut self) -> BumpAllocator {
        BumpAllocator::new(
            unsafe { NonNull::new_unchecked(self.raw_memory.as_mut_ptr()) },
            Self::memory_size(),
        )
    }
}

#[test]
fn bump_allocator_allocating_too_much_fails_with_out_of_memory() {
    let mut test = TestFixture::new();
    let sut = test.create_bump_allocator();

    let sample = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(TestFixture::memory_size() * 2, 1) });

    assert_that!(sample, is_err);
    assert_that!(sample.err().unwrap(), eq  AllocationError::OutOfMemory);
}

#[test]
fn bump_allocator_allocating_all_memory_works() {
    let mut test = TestFixture::new();
    let sut = test.create_bump_allocator();

    let number_of_samples = 8;
    let sample_size = TestFixture::memory_size() / number_of_samples;
    for i in 0..number_of_samples {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
        assert_that!(sample, is_ok);
        assert_that!(
            (sample.unwrap().as_ptr() as *mut u8) as u64, eq
            test.raw_memory.as_ptr() as u64 + (i * sample_size) as u64
        );
    }

    let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
    assert_that!(sample, is_err);
    assert_that!(sample.err().unwrap(), eq AllocationError::OutOfMemory);
}

#[test]
fn bump_allocator_after_deallocate_allocating_all_memory_works() {
    let mut test = TestFixture::new();
    let sut = test.create_bump_allocator();

    let number_of_samples = 8;
    let sample_size = TestFixture::memory_size() / number_of_samples;
    for _ in 0..number_of_samples {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
        assert_that!(sample, is_ok);
    }

    unsafe {
        sut.deallocate(
            NonNull::new_unchecked(test.raw_memory.as_mut_ptr()),
            Layout::from_size_align_unchecked(1, 1),
        );
    }

    for _ in 0..number_of_samples {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
        assert_that!(sample, is_ok);
    }
}

#[test]
fn bump_allocator_used_free_and_total_space_work() {
    let mut test = TestFixture::new();
    let sut = test.create_bump_allocator();

    let mut space_used = 331;
    while space_used < TestFixture::memory_size() {
        assert_that!(
            sut.allocate(unsafe { Layout::from_size_align_unchecked(331, 1) }),
            is_ok
        );

        assert_that!(sut.used_space(), eq space_used);
        assert_that!(sut.free_space(), eq TestFixture::memory_size() - space_used);
        assert_that!(sut.total_space(), eq TestFixture::memory_size());
        space_used += 331;
    }
}

#[test]
fn bump_allocator_allocating_with_different_alignments_works() {
    let mut test = TestFixture::new();
    let sut = test.create_bump_allocator();

    for i in [
        [32, 8],
        [1, 64],
        [1, 1],
        [2, 1],
        [5, 16],
        [200, 128],
        [129, 256],
    ] {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(i[0], i[1]) });
        assert_that!(sample, is_ok);
        let sample_addr = (sample.unwrap().as_ptr() as *const u8) as usize;
        assert_that!(sample_addr, mod i[1], is 0);
    }
}
