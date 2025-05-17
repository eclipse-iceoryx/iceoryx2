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

use iceoryx2_bb_elementary::math::*;
use iceoryx2_bb_elementary_traits::allocator::*;
use iceoryx2_bb_memory::one_chunk_allocator::*;
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
        1024
    }

    fn get_mut_memory(&mut self) -> *mut u8 {
        self.raw_memory.as_mut_ptr()
    }

    fn get_memory(&mut self) -> *const u8 {
        self.raw_memory.as_ptr()
    }

    fn create_one_chunk_allocator(&mut self) -> OneChunkAllocator {
        OneChunkAllocator::new(
            NonNull::new(self.get_mut_memory()).unwrap(),
            TestFixture::memory_size(),
        )
    }
}

#[test]
fn one_chunk_allocator_acquire_works() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;

    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");
    assert_that!(unsafe { memory.as_ref() }, len TestFixture::memory_size());
    assert_that!(
        unsafe { memory.as_ref() }.as_ptr() as usize, eq
        test.get_memory() as usize
    );
}

#[test]
fn one_chunk_allocator_acquire_with_alignment_works() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 256;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");
    let start = test.get_memory() as usize;
    let aligned_start = align(start, CHUNK_ALIGNMENT);
    assert_that!(
        unsafe { memory.as_ref() }.len(), eq
        TestFixture::memory_size() - (aligned_start - start)
    );
    assert_that!(unsafe { memory.as_ref() }.as_ptr() as usize, eq aligned_start);
}

#[test]
fn one_chunk_allocator_allocate_zeroed_works() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let memory = sut
        .allocate_zeroed(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    for i in 0..CHUNK_SIZE {
        assert_that!(unsafe { memory.as_ref().to_vec()[i] }, eq 0);
    }
}

#[test]
fn one_chunk_allocator_shrink_works() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    let memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    assert_that!(unsafe { memory.as_ref() }, len CHUNK_SIZE / 2);
}

#[test]
fn one_chunk_allocator_shrink_fails_when_size_increases() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    assert_that!(
        unsafe {
            sut.shrink(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
                Layout::from_size_align_unchecked(CHUNK_SIZE * 2, CHUNK_ALIGNMENT),
            )
        },
        is_err
    );
}

#[test]
fn one_chunk_allocator_shrink_fails_when_alignment_increases() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    assert_that!(
        unsafe {
            sut.shrink(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
                Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT * 2),
            )
        },
        is_err
    );
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn one_chunk_allocator_shrink_non_allocated_chunk_fails() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    unsafe {
        let _ = sut.shrink(
            NonNull::new(1234 as *mut u8).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
        );
    }
}

#[test]
fn one_chunk_allocator_grow_works() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    let mut memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    let memory = unsafe {
        sut.grow(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    assert_that!(unsafe { memory.as_ref() }, len TestFixture::memory_size());
}

#[test]
fn one_chunk_allocator_grow_zeroed_works() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    let mut memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    let memory = unsafe {
        sut.grow_zeroed(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    for i in 0..CHUNK_SIZE / 2 {
        assert_that!(unsafe { memory.as_ref() }.to_vec()[i], eq 255);
    }

    for i in CHUNK_SIZE / 2..CHUNK_SIZE {
        assert_that!(unsafe { memory.as_ref() }.to_vec()[i], eq 0);
    }

    assert_that!(unsafe { memory.as_ref() }, len TestFixture::memory_size());
}

#[test]
fn one_chunk_allocator_grow_with_decreased_size_fails() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    let mut memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    assert_that!(
        unsafe {
            sut.grow(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
                Layout::from_size_align_unchecked(CHUNK_SIZE / 4, CHUNK_ALIGNMENT),
            )
        },
        is_err
    );
}

#[test]
fn one_chunk_allocator_grow_with_increased_alignment_fails() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) })
        .expect("");

    let mut memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
        )
        .expect("")
    };

    assert_that!(
        unsafe {
            sut.grow(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
                Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT * 2),
            )
        },
        is_err
    );
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn one_chunk_allocator_grow_with_non_allocated_chunk_fails() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    unsafe {
        let _ = sut.grow(
            NonNull::new(123 as *mut u8).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE / 2, CHUNK_ALIGNMENT),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
        );
    }
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn one_chunk_allocator_deallocate_non_allocated_chunk_fails() {
    const CHUNK_SIZE: usize = 128;
    const CHUNK_ALIGNMENT: usize = 1;
    let mut test = TestFixture::new();
    let sut = test.create_one_chunk_allocator();

    assert_that!(
        sut.allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT) }),
        is_ok
    );

    unsafe {
        sut.deallocate(
            NonNull::new(123 as *mut u8).unwrap(),
            Layout::from_size_align_unchecked(CHUNK_SIZE, CHUNK_ALIGNMENT),
        );
    }
}
