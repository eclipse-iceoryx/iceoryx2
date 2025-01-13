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

use core::{alloc::Layout, ptr::NonNull};

use iceoryx2_bb_memory::heap_allocator::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn heap_allocator_allocate_deallocate_works() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let sut = HeapAllocator::new();
    let mut memory = sut.allocate(layout).unwrap();

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { memory.as_mut()[i] = 255 };
    }

    unsafe {
        sut.deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };
}

#[test]
fn heap_allocator_allocating_memory_with_size_of_zero_fails() {
    const MEM_SIZE: usize = 0;
    const MEM_ALIGN: usize = 256;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    assert_that!(sut.allocate(layout), is_err);
}

#[test]
fn heap_allocator_allocate_zeroed_and_free_works() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let sut = HeapAllocator::new();
    let memory = sut.allocate_zeroed(layout).unwrap();

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        assert_that!(unsafe { memory.as_ref()[i] }, eq 0)
    }

    unsafe {
        sut.deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };
}

#[test]
fn heap_allocator_allocating_zeroed_memory_with_size_of_zero_fails() {
    const MEM_SIZE: usize = 0;
    const MEM_ALIGN: usize = 8;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    assert_that!(sut.allocate_zeroed(layout), is_err);
}

#[test]
fn heap_allocator_grow_memory_keeps_content() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = sut.allocate(layout).unwrap();

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { memory.as_mut()[i] = 255 };
    }

    // resize
    let memory = unsafe {
        sut.grow(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
            Layout::from_size_align_unchecked(MEM_SIZE * 3, MEM_ALIGN),
        )
        .unwrap()
    };
    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE * 3);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        assert_that!(unsafe { memory.as_ref()[i] }, eq 255)
    }

    unsafe {
        sut.deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };
}

#[test]
fn heap_allocator_shrink_memory_keeps_content() {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = sut.allocate(layout).unwrap();

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { memory.as_mut()[i] = 255 };
    }

    // resize
    let memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
            Layout::from_size_align_unchecked(MEM_SIZE / 2, MEM_ALIGN),
        )
        .unwrap()
    };
    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE / 2);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE / 2 {
        assert_that!(unsafe { memory.as_ref()[i] }, eq 255)
    }

    unsafe {
        sut.deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };
}

#[test]
fn heap_allocator_shrink_memory_to_zero_fails() -> Result<(), AllocationError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = sut.allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    // resize
    assert_that!(
        unsafe {
            sut.shrink(
                NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
                layout,
                Layout::from_size_align_unchecked(0, MEM_ALIGN),
            )
        },
        is_err
    );
    Ok(())
}

#[test]
fn heap_allocator_grow_memory_with_increased_alignment_fails() -> Result<(), AllocationError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let sut = HeapAllocator::new();
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = sut.allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    assert_that!(
        unsafe {
            sut.grow(
                NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
                layout,
                Layout::from_size_align_unchecked(MEM_SIZE * 2, MEM_ALIGN * 2),
            )
        },
        is_err
    );
    Ok(())
}
