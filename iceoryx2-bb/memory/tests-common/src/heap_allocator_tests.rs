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

use core::alloc::Layout;

use iceoryx2_bb_elementary_traits::pointer::Pointer;
use iceoryx2_bb_memory::{
    heap_allocator::*,
    pool_allocator::{ReallocGrow, ReallocShrink},
};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn allocate_deallocate_works() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let sut = HeapAllocator::new();
    let memory = sut.allocate(layout).unwrap();

    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { *memory.as_ptr().add(i) = 255 };
    }

    unsafe { sut.deallocate(memory, layout) };
}

#[test]
pub fn allocating_memory_with_size_of_zero_fails() {
    const MEM_SIZE: usize = 0;
    const MEM_ALIGN: usize = 256;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    assert_that!(sut.allocate(layout), is_err);
}

#[test]
pub fn allocate_zeroed_and_free_works() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let sut = HeapAllocator::new();
    let memory = sut.allocate_zeroed(layout).unwrap();

    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        assert_that!(unsafe { *memory.as_ptr().add(i) }, eq 0)
    }

    unsafe { sut.deallocate(memory, layout) };
}

#[test]
pub fn allocating_zeroed_memory_with_size_of_zero_fails() {
    const MEM_SIZE: usize = 0;
    const MEM_ALIGN: usize = 8;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    assert_that!(sut.allocate_zeroed(layout), is_err);
}

#[test]
pub fn grow_memory_keeps_content() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = sut.allocate(layout).unwrap();

    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { *memory.as_mut_ptr().add(i) = 255 };
    }

    // resize
    let memory = unsafe {
        sut.grow(
            memory,
            layout,
            Layout::from_size_align_unchecked(MEM_SIZE * 3, MEM_ALIGN),
        )
        .unwrap()
    };
    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        assert_that!(unsafe { *memory.as_ptr().add(i) }, eq 255)
    }

    unsafe { sut.deallocate(memory, layout) };
}

#[test]
pub fn shrink_memory_keeps_content() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = sut.allocate(layout).unwrap();

    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { *memory.as_mut_ptr().add(i) = 255 };
    }

    // resize
    let memory = unsafe {
        sut.shrink(
            memory,
            layout,
            Layout::from_size_align_unchecked(MEM_SIZE / 2, MEM_ALIGN),
        )
        .unwrap()
    };
    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE / 2 {
        assert_that!(unsafe { *memory.as_ptr().add(i) }, eq 255)
    }

    unsafe { sut.deallocate(memory, layout) };
}

#[test]
pub fn shrink_memory_to_zero_fails() -> Result<(), AllocationError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = sut.allocate(layout)?;

    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    // resize
    assert_that!(
        unsafe {
            sut.shrink(
                memory,
                layout,
                Layout::from_size_align_unchecked(0, MEM_ALIGN),
            )
        },
        is_err
    );
    Ok(())
}

#[test]
pub fn grow_memory_with_increased_alignment_fails() -> Result<(), AllocationError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = sut.allocate(layout)?;

    let address = memory.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    assert_that!(
        unsafe {
            sut.grow(
                memory,
                layout,
                Layout::from_size_align_unchecked(MEM_SIZE * 2, MEM_ALIGN * 2),
            )
        },
        is_err
    );
    Ok(())
}
