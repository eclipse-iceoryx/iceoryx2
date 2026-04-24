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

use core::{alloc::Layout, ptr::NonNull};

use iceoryx2_bb_elementary::{bump_allocator::*, math::align};
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn start_position_is_correctly_used() {
    let mut memory = [0u8; 8192];
    let start_position: *mut u8 = memory.as_mut_ptr();
    const MEM_SIZE: usize = 91;
    const MEM_ALIGN: usize = 1;
    let sut = BumpAllocator::new(start_position);

    let memory = sut
        .allocate(Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap())
        .unwrap();

    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq start_position as usize);
    assert_that!(unsafe { memory.as_ref() }.len(), eq MEM_SIZE);
}

#[test]
pub fn allocated_memory_is_correctly_aligned() {
    let mut memory = [0u8; 8192];
    let start_position: *mut u8 = unsafe { memory.as_mut_ptr().add(1) };
    const MEM_SIZE: usize = 128;
    const MEM_ALIGN: usize = 64;
    let sut = BumpAllocator::new(start_position);

    let memory = sut
        .allocate(Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap())
        .unwrap();

    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq align(start_position as usize, MEM_ALIGN));
    assert_that!(unsafe { memory.as_ref() }.len(), eq MEM_SIZE);
}

#[test]
pub fn allocating_many_aligned_chunks_work() {
    let mut memory = [0u8; 8192];
    let start_position: *mut u8 = unsafe { memory.as_mut_ptr().add(1) };
    const ITERATIONS: u32 = 5;
    let sut = BumpAllocator::new(start_position);

    let mut last_size = 0;
    let mut last_position = 0;
    for n in 0..ITERATIONS {
        let mem_size = 4_usize.pow(n);
        let mem_align = 2_usize.pow(n);
        let memory = sut
            .allocate(Layout::from_size_align(mem_size, mem_align).unwrap())
            .unwrap();

        let new_position = unsafe { memory.as_ref() }.as_ptr() as usize;
        assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq align(new_position, mem_align));
        assert_that!(unsafe { memory.as_ref() }.len(), eq mem_size);
        assert_that!(new_position - last_position, ge last_size);

        last_position = new_position;
        last_size = mem_size;
    }
}

#[test]
pub fn deallocating_releases_everything() {
    let mut memory = [0u8; 8192];
    let start_position: *mut u8 = unsafe { memory.as_mut_ptr().add(3) };
    const MEM_SIZE: usize = 128;
    const MEM_ALIGN: usize = 1;
    let sut = BumpAllocator::new(start_position);

    let layout = Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap();
    let mut memory = sut.allocate(layout).unwrap();

    unsafe {
        sut.deallocate(
            NonNull::new(memory.as_mut().as_mut_ptr().cast()).unwrap(),
            layout,
        )
    };

    let memory = sut
        .allocate(Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap())
        .unwrap();

    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq start_position as usize);
    assert_that!(unsafe { memory.as_ref() }.len(), eq MEM_SIZE);
}

#[test]
pub fn allocating_too_much_fails_with_out_of_memory() {
    let memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        <NonNull<u8> as NonNullCompat<u8>>::from_ref(&memory[0]),
        memory.len(),
    );

    let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(8192 * 2, 1) });

    assert_that!(sample, is_err);
    assert_that!(sample.err().unwrap(), eq  AllocationError::OutOfMemory);
}

#[test]
pub fn allocating_all_memory_works() {
    let memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        <NonNull<u8> as NonNullCompat<u8>>::from_ref(&memory[0]),
        memory.len(),
    );

    let number_of_samples = 8;
    let sample_size = 8192 / number_of_samples;
    for i in 0..number_of_samples {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
        assert_that!(sample, is_ok);
        assert_that!(
            (sample.unwrap().as_ptr() as *mut u8) as u64, eq
            memory.as_ptr() as u64 + (i * sample_size) as u64
        );
    }

    let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
    assert_that!(sample, is_err);
    assert_that!(sample.err().unwrap(), eq AllocationError::OutOfMemory);
}

#[test]
pub fn after_deallocate_allocating_all_memory_works() {
    let mut memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        <NonNull<u8> as NonNullCompat<u8>>::from_ref(&memory[0]),
        memory.len(),
    );

    let number_of_samples = 8;
    let sample_size = 8192 / number_of_samples;
    for _ in 0..number_of_samples {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
        assert_that!(sample, is_ok);
    }

    unsafe {
        sut.deallocate(
            NonNull::new_unchecked(memory.as_mut_ptr()),
            Layout::from_size_align_unchecked(1, 1),
        );
    }

    for _ in 0..number_of_samples {
        let sample = sut.allocate(unsafe { Layout::from_size_align_unchecked(sample_size, 1) });
        assert_that!(sample, is_ok);
    }
}

#[test]
pub fn used_free_and_total_space_work() {
    let memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        <NonNull<u8> as NonNullCompat<u8>>::from_ref(&memory[0]),
        memory.len(),
    );

    let mut space_used = 331;
    while space_used < 8192 {
        assert_that!(
            sut.allocate(unsafe { Layout::from_size_align_unchecked(331, 1) }),
            is_ok
        );

        assert_that!(sut.used_space(), eq space_used);
        assert_that!(sut.free_space(), eq 8192 - space_used);
        assert_that!(sut.total_space(), eq 8192);
        space_used += 331;
    }
}

#[test]
pub fn allocating_with_different_alignments_works() {
    let memory = [0u8; 8192];
    let sut = BumpAllocator::new(
        <NonNull<u8> as NonNullCompat<u8>>::from_ref(&memory[0]),
        memory.len(),
    );

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
