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

use iceoryx2_bb_elementary::math::align;
use iceoryx2_bb_elementary_traits::allocator::*;
use iceoryx2_bb_memory::{bump_allocator::BumpAllocator, pool_allocator::*};
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

    fn calc_min_number_of_buckets(size: usize, align: usize) -> u32 {
        (TestFixture::memory_size() / core::cmp::max(size, align)) as u32 - 1
    }

    fn create_pool_allocator(&mut self, size: usize, align: usize) -> FixedSizePoolAllocator<512> {
        FixedSizePoolAllocator::<512>::new(
            unsafe { Layout::from_size_align_unchecked(size, align) },
            NonNull::new(self.get_mut_memory()).unwrap(),
            TestFixture::memory_size(),
        )
    }
}

#[test]
fn pool_allocator_set_up_correctly() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    assert_that!(sut.bucket_size(), eq BUCKET_SIZE);
    assert_that!(sut.max_alignment(), eq BUCKET_ALIGNMENT);
    assert_that!(sut.number_of_buckets() as usize, le TestFixture::memory_size() / BUCKET_SIZE);
}

#[test]
fn pool_allocator_acquire_all_memory_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 8;
    const CHUNK_SIZE: usize = 100;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    assert_that!(
        sut.number_of_buckets()
            , ge TestFixture::calc_min_number_of_buckets(BUCKET_SIZE, BUCKET_ALIGNMENT)
    );

    let start_addr = align(test.get_memory() as usize, BUCKET_ALIGNMENT);
    for i in 0..sut.number_of_buckets() {
        let memory = sut
            .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) })
            .expect("");

        let addr = start_addr + i as usize * BUCKET_SIZE;
        assert_that!((unsafe { memory.as_ref() }.as_ptr()) as usize, eq addr);
        assert_that!(addr, mod BUCKET_ALIGNMENT, is 0);
        assert_that!(unsafe { memory.as_ref() }, len CHUNK_SIZE);
    }

    let memory = sut.allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) });
    assert_that!(memory, is_err);
}

#[test]
fn pool_allocator_allocate_more_than_bucket_size_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 8;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    assert_that!(
        sut.allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE + 1, 1) }),
        is_err
    );
}

#[test]
fn pool_allocator_allocate_more_than_bucket_alignment_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 8;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    assert_that!(
        sut.allocate(unsafe {
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT * 2)
        }),
        is_err
    );
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn pool_allocator_deallocate_non_allocated_chunk_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 8;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    assert_that!(
        sut.allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT) }),
        is_ok
    );

    unsafe {
        sut.deallocate(
            NonNull::new(123 as *mut u8).unwrap(),
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
        );
    }
}

#[test]
fn pool_allocator_acquire_and_release_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 8;
    const BUCKET_ALIGNMENT: usize = 128;
    const CHUNK_SIZE: usize = 5;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);
    let mut memory_storage = vec![];

    assert_that!(sut.number_of_buckets(), ge 7);
    for _ in 0..sut.number_of_buckets() {
        let memory = sut
            .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) })
            .expect("");
        assert_that!(unsafe { memory.as_ref() }, len CHUNK_SIZE);
        memory_storage.push(memory);
    }
    let memory = sut.allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) });
    assert_that!(memory, is_err);

    for memory in memory_storage {
        unsafe {
            sut.deallocate(
                NonNull::new(memory.as_ref().as_ptr() as *mut u8).unwrap(),
                Layout::from_size_align_unchecked(CHUNK_SIZE, 1),
            );
        }
    }

    for _ in 0..sut.number_of_buckets() {
        let memory = sut
            .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE + 2, 1) })
            .expect("");
        assert_that!(unsafe { memory.as_ref() }, len CHUNK_SIZE + 2);
    }
    let memory = sut.allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) });
    assert_that!(memory, is_err);
}

#[test]
fn pool_allocator_acquire_too_large_sample_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);
    assert_that!(
        sut.allocate(unsafe {
            Layout::from_size_align_unchecked(BUCKET_SIZE + 1, BUCKET_ALIGNMENT)
        }),
        is_err
    );
}

#[test]
fn pool_allocator_acquire_sample_with_to_large_alignment_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    assert_that!(
        sut.allocate(unsafe {
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT * 2)
        }),
        is_err
    );
}

#[test]
fn pool_allocator_allocate_zeroed_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let memory = sut
        .allocate_zeroed(unsafe {
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT)
        })
        .expect("failed to get memory");

    for i in 0..BUCKET_SIZE {
        assert_that!(unsafe { memory.as_ref().to_vec()[i] }, eq 0);
    }
}

#[test]
fn pool_allocator_grow_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");
    assert_that!(unsafe { memory.as_ref() }, len BUCKET_SIZE / 2);

    let memory = unsafe {
        sut.grow(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
        )
        .expect("")
    };
    assert_that!(unsafe { memory.as_ref() }, len BUCKET_SIZE);
}

#[test]
fn pool_allocator_grow_with_size_larger_bucket_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    assert_that!(
        unsafe {
            sut.grow(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
                Layout::from_size_align_unchecked(BUCKET_SIZE + 1, BUCKET_ALIGNMENT),
            )
        },
        is_err
    );
}

#[test]
fn pool_allocator_grow_with_size_decrease_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    assert_that!(
        unsafe {
            sut.grow(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
                Layout::from_size_align_unchecked(BUCKET_SIZE / 4, BUCKET_ALIGNMENT),
            )
        },
        is_err
    );
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn pool_allocator_grow_with_non_allocated_chunk_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    unsafe {
        let _ = sut.grow(
            NonNull::new(431 as *mut u8).unwrap(),
            Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
        );
    }
}

#[test]
fn pool_allocator_grow_with_too_alignment_larger_bucket_alignment_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    assert_that!(
        unsafe {
            sut.grow(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
                Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT * 8),
            )
        },
        is_err
    );
}

#[test]
fn pool_allocator_grow_zeroed_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    let memory = unsafe {
        sut.grow_zeroed(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
        )
        .expect("")
    };

    for i in 0..BUCKET_SIZE / 2 {
        assert_that!(unsafe { memory.as_ref() }.to_vec()[i], eq 255);
    }

    for i in BUCKET_SIZE / 2..BUCKET_SIZE {
        assert_that!(unsafe { memory.as_ref() }.to_vec()[i], eq 0);
    }
}

#[test]
fn pool_allocator_shrink_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    let memory = unsafe {
        sut.shrink(
            NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
            Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
        )
        .expect("")
    };

    assert_that!(unsafe { memory.as_ref() }, len BUCKET_SIZE / 2);
}

#[test]
fn pool_allocator_shrink_with_increased_size_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    assert_that!(
        unsafe {
            sut.shrink(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
                Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
            )
        },
        is_err
    );
}

#[test]
fn pool_allocator_shrink_with_alignment_larger_than_bucket_alignment_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    let mut memory = sut
        .allocate(unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT) })
        .expect("failed to get memory");

    assert_that!(
        unsafe {
            sut.shrink(
                NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
                Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
                Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT * 32),
            )
        },
        is_err
    );
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn pool_allocator_shrink_non_allocated_chunk_fails() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 1;

    let sut = test.create_pool_allocator(BUCKET_SIZE, BUCKET_ALIGNMENT);

    unsafe {
        let _ = sut.shrink(
            NonNull::new(1234 as *mut u8).unwrap(),
            Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT),
            Layout::from_size_align_unchecked(BUCKET_SIZE / 2, BUCKET_ALIGNMENT),
        );
    };
}

#[test]
fn pool_allocator_relocatable_acquire_all_memory_works() {
    let mut test = TestFixture::new();
    const BUCKET_SIZE: usize = 128;
    const BUCKET_ALIGNMENT: usize = 8;
    const BUCKET_LAYOUT: Layout =
        unsafe { Layout::from_size_align_unchecked(BUCKET_SIZE, BUCKET_ALIGNMENT) };
    const CHUNK_SIZE: usize = 100;
    let start_ptr = NonNull::new(test.get_mut_memory()).unwrap();

    let mut sut =
        unsafe { PoolAllocator::new_uninit(BUCKET_LAYOUT, start_ptr, TestFixture::memory_size()) };

    let mgmt_memory_size = PoolAllocator::memory_size(BUCKET_LAYOUT, TestFixture::memory_size());
    let mut mgmt_memory = Vec::<u8>::new();
    mgmt_memory.resize(mgmt_memory_size + 100, 137u8);
    let mgmt_memory_slice = mgmt_memory.as_mut_slice();
    let bump_allocator = BumpAllocator::new(
        NonNull::new(mgmt_memory_slice.as_mut_ptr()).unwrap(),
        mgmt_memory_slice.len(),
    );

    for i in mgmt_memory_size..mgmt_memory_size + 100 {
        assert_that!(mgmt_memory[i], eq 137u8);
    }

    assert_that!(unsafe { sut.init(&bump_allocator) }, is_ok);

    assert_that!(
        sut.number_of_buckets(), ge
             TestFixture::calc_min_number_of_buckets(BUCKET_SIZE, BUCKET_ALIGNMENT)
    );

    let start_addr = align(test.get_memory() as usize, BUCKET_ALIGNMENT);
    for i in 0..sut.number_of_buckets() {
        let memory = sut
            .allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) })
            .expect("");
        let addr = start_addr + i as usize * BUCKET_SIZE;
        assert_that!((unsafe { memory.as_ref() }.as_ptr()) as usize, eq addr);
        assert_that!(addr, mod BUCKET_ALIGNMENT, is 0);
        assert_that!(unsafe { memory.as_ref() }, len CHUNK_SIZE);
    }

    let memory = sut.allocate(unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) });
    assert_that!(memory, is_err);
}
