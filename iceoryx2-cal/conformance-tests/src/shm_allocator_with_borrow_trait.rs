// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod shm_allocator_with_borrow_trait {
    use core::{alloc::Layout, ptr::NonNull};
    use iceoryx2_bb_elementary_traits::allocator::{Allocate, Deallocate};
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::shm_allocator::{PointerOffset, ShmAllocator, *};

    const MEMORY_SIZE: usize = 4096;
    const MGMT_SIZE: usize = 4096;
    const CHUNK_SIZE: usize = 128;
    const MAX_NUMBER_OF_CHUNKS: usize = MEMORY_SIZE / CHUNK_SIZE;
    const MAX_ALIGNMENT: usize = 512;
    const SOME_OFFSET: usize = 13;

    struct Test<Sut: ShmAllocator> {
        memory: [u8; MEMORY_SIZE],
        mgmt_memory: [u8; MGMT_SIZE],
        sut: Option<Sut>,
    }

    impl<Sut: ShmAllocator> Test<Sut> {
        fn new() -> Self {
            Self {
                memory: [0u8; MEMORY_SIZE],
                mgmt_memory: [252u8; MGMT_SIZE],
                sut: None,
            }
        }

        fn sut(&mut self) -> &mut Sut {
            self.sut.as_mut().unwrap()
        }

        fn init(&mut self) {
            let required_mgmt_size =
                Sut::management_size(MEMORY_SIZE, &Sut::Configuration::default());
            let bump_allocator = BumpAllocator::new(
                NonNull::new(self.mgmt_memory.as_ptr() as *mut u8).unwrap(),
                MGMT_SIZE,
            );

            self.sut = Some(unsafe {
                Sut::new_uninit(
                    MAX_ALIGNMENT,
                    NonNull::new_unchecked(&mut self.memory.as_mut_slice()[SOME_OFFSET..4096]),
                    &Sut::Configuration::default(),
                )
            });

            assert_that!(unsafe { self.sut().init(&bump_allocator) }, is_ok);

            for i in required_mgmt_size..MGMT_SIZE {
                assert_that!(self.mgmt_memory[i], eq 252u8);
            }
        }

        fn layout(&self) -> Layout {
            unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) }
        }
    }

    #[conformance_test]
    pub fn release_deallocates_an_allocated_chunk<Sut: ShmAllocator>()
    where
        for<'a> Sut::Initialized<'a>: Borrowable,
    {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout = test.layout();

        for _ in 0..MAX_NUMBER_OF_CHUNKS * 2 {
            let ptr = unsafe { test.sut().assume_init().allocate(layout) };
            assert_that!(ptr, is_ok);
            unsafe { test.sut().assume_init().release(ptr.unwrap()) };
        }
    }

    #[conformance_test]
    pub fn borrowed_chunks_are_not_deallocated_when_not_every_instance_is_released<
        Sut: ShmAllocator,
    >()
    where
        for<'a> Sut::Initialized<'a>: Borrowable,
    {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout = test.layout();

        for _ in 0..MAX_NUMBER_OF_CHUNKS * 2 {
            let ptr = unsafe { test.sut().assume_init().allocate(layout) };
            if ptr.is_err() {
                break;
            }

            unsafe { test.sut().assume_init().borrow(ptr.unwrap()) };
            unsafe { test.sut().assume_init().borrow(ptr.unwrap()) };
            unsafe { test.sut().assume_init().borrow(ptr.unwrap()) };
            unsafe { test.sut().assume_init().release(ptr.unwrap()) };
        }

        assert_that!(unsafe { test.sut().assume_init().allocate(layout) }.err(), eq Some(AllocationError::OutOfMemory));
    }

    #[conformance_test]
    pub fn borrowed_chunks_are_deallocated_with_the_correct_amount_of_release_calls<
        Sut: ShmAllocator,
    >()
    where
        for<'a> Sut::Initialized<'a>: Borrowable,
    {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout = test.layout();

        for _ in 0..MAX_NUMBER_OF_CHUNKS * 2 {
            let ptr = unsafe { test.sut().assume_init().allocate(layout) };
            assert_that!(ptr, is_ok);
            unsafe { test.sut().assume_init().borrow(ptr.unwrap()) };
            unsafe { test.sut().assume_init().borrow(ptr.unwrap()) };

            assert_that!(unsafe { test.sut().assume_init().release(ptr.unwrap()) }, eq ReleaseState::Kept);
            assert_that!(unsafe { test.sut().assume_init().release(ptr.unwrap()) }, eq ReleaseState::Kept);
            assert_that!(unsafe { test.sut().assume_init().release(ptr.unwrap()) }, eq ReleaseState::Deallocated);
        }
    }

    #[conformance_test]
    pub fn explicit_deallocate_works_when_ownership_count_is_one<Sut: ShmAllocator>()
    where
        for<'a> Sut::Initialized<'a>: Borrowable + Deallocate<PointerOffset>,
    {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout = test.layout();

        for _ in 0..MAX_NUMBER_OF_CHUNKS * 2 {
            let ptr = unsafe { test.sut().assume_init().allocate(layout) };
            assert_that!(ptr, is_ok);
            let ptr = ptr.unwrap();
            unsafe { test.sut().assume_init().borrow(ptr) };
            unsafe { test.sut().assume_init().borrow(ptr) };

            unsafe { test.sut().assume_init().release(ptr) };
            unsafe { test.sut().assume_init().release(ptr) };

            unsafe { test.sut().assume_init().deallocate(ptr, layout) };
        }
    }
}
