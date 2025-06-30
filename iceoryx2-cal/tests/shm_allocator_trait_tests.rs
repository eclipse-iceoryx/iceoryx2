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

#[generic_tests::define]
mod shm_allocator {
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::shm_allocator::{ShmAllocator, *};

    use core::{alloc::Layout, ptr::NonNull};
    use std::{collections::HashSet, sync::Mutex};

    use lazy_static::lazy_static;

    const MEMORY_SIZE: usize = 4096;
    const MGMT_SIZE: usize = 4096;
    const CHUNK_SIZE: usize = 128;
    const MAX_ALIGNMENT: usize = 512;

    struct TestFixture<Sut: ShmAllocator> {
        memory: [u8; MEMORY_SIZE],
        mgmt_memory: [u8; MGMT_SIZE],
        required_mgmt_size: usize,
        bump_allocator: Option<BumpAllocator>,
        sut: Option<Sut>,
    }

    impl<Sut: ShmAllocator> TestFixture<Sut> {
        fn new() -> Self {
            Self {
                memory: [0u8; MEMORY_SIZE],
                mgmt_memory: [252u8; MGMT_SIZE],
                sut: None,
                required_mgmt_size: 0,
                bump_allocator: None,
            }
        }

        fn sut(&mut self) -> &mut Sut {
            self.sut.as_mut().unwrap()
        }

        fn bump_allocator(&self) -> &BumpAllocator {
            self.bump_allocator.as_ref().unwrap()
        }

        fn prepare(&mut self) {
            self.required_mgmt_size =
                Sut::management_size(MEMORY_SIZE, &Sut::Configuration::default());
            self.bump_allocator = Some(BumpAllocator::new(
                NonNull::new(self.mgmt_memory.as_ptr() as *mut u8).unwrap(),
                MGMT_SIZE,
            ));
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
                    NonNull::new_unchecked(&mut self.memory.as_mut_slice()[13..4096]),
                    &Sut::Configuration::default(),
                )
            });

            assert_that!(unsafe { self.sut().init(&bump_allocator) }, is_ok);

            for i in required_mgmt_size..MGMT_SIZE {
                assert_that!(self.mgmt_memory[i], eq 252u8);
            }
        }
    }

    #[test]
    fn allocate_and_free_works<Sut: ShmAllocator>() {
        let mut test = TestFixture::<Sut>::new();
        test.init();

        let layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
        let distance = unsafe { test.sut().allocate(layout) };
        assert_that!(distance, is_ok);

        unsafe {
            test.sut().deallocate(distance.unwrap(), layout);
        }
    }

    #[test]
    fn first_allocated_offset_must_start_at_zero<Sut: ShmAllocator>() {
        let mut test = TestFixture::<Sut>::new();
        test.init();

        let layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
        let distance = unsafe { test.sut().allocate(layout).unwrap() };
        assert_that!(distance.offset(), eq 0);

        unsafe { test.sut().deallocate(distance, layout) };
    }

    #[test]
    fn allocate_max_alignment_works<Sut: ShmAllocator>() {
        let mut test = TestFixture::<Sut>::new();
        test.init();

        let layout =
            unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, test.sut().max_alignment()) };
        let distance = unsafe { test.sut().allocate(layout) };
        assert_that!(distance, is_ok);

        unsafe {
            test.sut().deallocate(distance.unwrap(), layout);
        }
    }

    #[test]
    fn allocate_more_than_max_alignment_fails<Sut: ShmAllocator>() {
        let mut test = TestFixture::<Sut>::new();
        test.init();

        let layout = unsafe {
            Layout::from_size_align_unchecked(
                CHUNK_SIZE,
                (test.sut().max_alignment() + 1).next_power_of_two(),
            )
        };
        let distance = unsafe { test.sut().allocate(layout) };
        assert_that!(distance, is_err);
        assert_that!(
            distance.err().unwrap(), eq
            ShmAllocationError::ExceedsMaxSupportedAlignment
        );
    }

    #[test]
    fn init_fails_when_supported_memory_alignment_is_smaller_than_required<Sut: ShmAllocator>() {
        let mut test = TestFixture::<Sut>::new();
        test.prepare();

        let mut sut = unsafe {
            Sut::new_uninit(
                1,
                NonNull::new_unchecked(test.memory.as_mut_slice()),
                &Sut::Configuration::default(),
            )
        };

        let result = unsafe { sut.init(test.bump_allocator()) };
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            ShmAllocatorInitError::MaxSupportedMemoryAlignmentInsufficient
        );
    }

    #[test]
    fn allocator_id_is_unique<Sut: ShmAllocator>() {
        lazy_static! {
            static ref ALLOCATOR_IDS: Mutex<HashSet<u8>> = Mutex::new(HashSet::new());
        }

        let uid = Sut::unique_id();
        let mut guard = ALLOCATOR_IDS.lock().unwrap();
        assert_that!(!guard.contains(&uid), eq true);
        guard.insert(uid);
    }

    #[instantiate_tests(<iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator>)]
    mod pool_allocator {}

    #[instantiate_tests(<iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator>)]
    mod bump_allocator {}
}
