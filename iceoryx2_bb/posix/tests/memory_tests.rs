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

use iceoryx2_bb_posix::memory::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn memory_allocate_and_deallocate_works() -> Result<(), MemoryError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = heap::allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { memory.as_mut()[i] = 255 };
    }

    unsafe {
        heap::deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };

    Ok(())
}

#[test]
fn memory_allocating_memory_with_size_of_zero_fails() {
    const MEM_SIZE: usize = 0;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    assert_that!(heap::allocate(layout), is_err);
}

#[test]
fn memory_allocate_zeroed_and_free_works() -> Result<(), MemoryError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 256;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = heap::allocate_zeroed(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        assert_that!(unsafe { memory.as_ref()[i] }, eq 0)
    }

    unsafe {
        heap::deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };

    Ok(())
}

#[test]
fn memory_allocating_zeroed_memory_with_size_of_zero_fails() {
    const MEM_SIZE: usize = 0;
    const MEM_ALIGN: usize = 8;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    assert_that!(heap::allocate_zeroed(layout), is_err);
}

#[test]
fn memory_increasing_memory_keeps_content() -> Result<(), MemoryError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = heap::allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { memory.as_mut()[i] = 255 };
    }

    // resize
    let memory = unsafe {
        heap::resize(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
            Layout::from_size_align_unchecked(MEM_SIZE * 3, MEM_ALIGN),
        )?
    };
    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE * 3);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        assert_that!(unsafe { memory.as_ref()[i] }, eq 255)
    }

    unsafe {
        heap::deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };

    Ok(())
}

#[test]
fn memory_decreasing_memory_keeps_content() -> Result<(), MemoryError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let mut memory = heap::allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE {
        unsafe { memory.as_mut()[i] = 255 };
    }

    // resize
    let memory = unsafe {
        heap::resize(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
            Layout::from_size_align_unchecked(MEM_SIZE / 2, MEM_ALIGN),
        )?
    };
    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE / 2);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    for i in 0..MEM_SIZE / 2 {
        assert_that!(unsafe { memory.as_ref()[i] }, eq 255)
    }

    unsafe {
        heap::deallocate(
            NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
            layout,
        )
    };

    Ok(())
}

#[test]
fn memory_decreasing_memory_to_zero_fails() -> Result<(), MemoryError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = heap::allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    // resize
    assert_that!(
        unsafe {
            heap::resize(
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
fn memory_resize_memory_with_increased_alignment_fails() -> Result<(), MemoryError> {
    const MEM_SIZE: usize = 1024;
    const MEM_ALIGN: usize = 1;
    let layout = unsafe { Layout::from_size_align_unchecked(MEM_SIZE, MEM_ALIGN) };
    let memory = heap::allocate(layout)?;

    assert_that!(unsafe { memory.as_ref() }, len MEM_SIZE);
    let address = unsafe { memory.as_ref() }.as_ptr() as usize;
    assert_that!(address, mod MEM_ALIGN, is 0);

    // resize
    assert_that!(
        unsafe {
            heap::resize(
                NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
                layout,
                Layout::from_size_align_unchecked(MEM_SIZE * 2, MEM_ALIGN * 2),
            )
        },
        is_err
    );
    Ok(())
}
