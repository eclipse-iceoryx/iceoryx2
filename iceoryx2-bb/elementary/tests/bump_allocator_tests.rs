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

#[test]
fn start_position_is_correctly_used() {
    const START_POSITION: usize = 918243;
    const MEM_SIZE: usize = 91;
    const MEM_ALIGN: usize = 1;
    let sut = BumpAllocator::new(START_POSITION as *mut u8);

    let memory = sut
        .allocate(Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap())
        .unwrap();

    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq START_POSITION);
    assert_that!(unsafe { memory.as_ref() }.len() as usize, eq MEM_SIZE);
}

#[test]
fn allocated_memory_is_correctly_aligned() {
    const START_POSITION: usize = 918243;
    const MEM_SIZE: usize = 128;
    const MEM_ALIGN: usize = 64;
    let sut = BumpAllocator::new(START_POSITION as *mut u8);

    let memory = sut
        .allocate(Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap())
        .unwrap();

    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq align(START_POSITION, MEM_ALIGN));
    assert_that!(unsafe { memory.as_ref() }.len() as usize, eq MEM_SIZE);
}

#[test]
fn allocating_many_aligned_chunks_work() {
    const ITERATIONS: u32 = 5;
    const START_POSITION: usize = 192874901;
    let sut = BumpAllocator::new(START_POSITION as *mut u8);

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
        assert_that!(unsafe { memory.as_ref() }.len() as usize, eq mem_size);
        assert_that!(new_position - last_position, ge last_size);

        last_position = new_position;
        last_size = mem_size;
    }
}

#[test]
fn deallocating_releases_everything() {
    const START_POSITION: usize = 918243;
    const MEM_SIZE: usize = 128;
    const MEM_ALIGN: usize = 1;
    let sut = BumpAllocator::new(START_POSITION as *mut u8);

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

    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq START_POSITION);
    assert_that!(unsafe { memory.as_ref() }.len() as usize, eq MEM_SIZE);
}
