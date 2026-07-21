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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod shm_allocator_trait {
    use alloc::collections::btree_set::BTreeSet;
    use core::{alloc::Layout, ptr::NonNull};
    use iceoryx2_bb_concurrency::lazy_lock::LazyLock;
    use iceoryx2_bb_elementary_traits::allocator::ContentPlacement;
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_posix::ipc_capable::Handle;
    use iceoryx2_bb_posix::mutex::{Mutex, MutexBuilder, MutexHandle};
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::shm_allocator::{ShmAllocator, *};

    const MEMORY_SIZE: usize = 4096;
    const MGMT_SIZE: usize = 4096;
    const CHUNK_SIZE: usize = 128;
    const MAX_ALIGNMENT: usize = 512;
    const SOME_OFFSET: usize = 13;

    struct Test<Sut: ShmAllocator> {
        memory: [u8; MEMORY_SIZE],
        mgmt_memory: [u8; MGMT_SIZE],
        required_mgmt_size: usize,
        bump_allocator: Option<BumpAllocator>,
        sut: Option<Sut>,
    }

    impl<Sut: ShmAllocator> Test<Sut> {
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
                    NonNull::new_unchecked(&mut self.memory.as_mut_slice()[SOME_OFFSET..4096]),
                    &Sut::Configuration::default(),
                )
            });

            assert_that!(unsafe { self.sut().init(&bump_allocator) }, is_ok);

            for i in required_mgmt_size..MGMT_SIZE {
                assert_that!(self.mgmt_memory[i], eq 252u8);
            }
        }

        fn offset_to_ptr(&mut self, offset: PointerOffset) -> *mut u8 {
            unsafe {
                self.memory
                    .as_mut()
                    .as_mut_ptr()
                    .add(offset.offset() + SOME_OFFSET + self.sut().relative_start_address())
            }
        }
    }

    #[conformance_test]
    pub fn allocate_and_free_works<Sut: ShmAllocator>() {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
        let distance = unsafe { test.sut().allocate(layout) };
        assert_that!(distance, is_ok);

        unsafe {
            test.sut().deallocate(distance.unwrap(), layout);
        }
    }

    #[conformance_test]
    pub fn first_allocated_offset_must_start_at_zero<Sut: ShmAllocator>() {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
        let distance = unsafe { test.sut().allocate(layout).unwrap() };
        assert_that!(distance.offset(), eq 0);

        unsafe { test.sut().deallocate(distance, layout) };
    }

    #[conformance_test]
    pub fn allocate_max_alignment_works<Sut: ShmAllocator>() {
        let mut test = Test::<Sut>::new();
        test.init();

        let layout =
            unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, test.sut().max_alignment()) };
        let distance = unsafe { test.sut().allocate(layout) };
        assert_that!(distance, is_ok);

        unsafe {
            test.sut().deallocate(distance.unwrap(), layout);
        }
    }

    #[conformance_test]
    pub fn allocate_more_than_max_alignment_fails<Sut: ShmAllocator>() {
        let mut test = Test::<Sut>::new();
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

    #[conformance_test]
    pub fn init_fails_when_supported_memory_alignment_is_smaller_than_required<
        Sut: ShmAllocator,
    >() {
        let mut test = Test::<Sut>::new();
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

    #[conformance_test]
    pub fn allocator_id_is_unique<Sut: ShmAllocator>() {
        static MTX_HANDLE: LazyLock<MutexHandle<BTreeSet<u8>>> = LazyLock::new(MutexHandle::new);
        static ALLOCATOR_IDS: LazyLock<Mutex<'static, 'static, BTreeSet<u8>>> =
            LazyLock::new(|| {
                MutexBuilder::new()
                    .create(BTreeSet::new(), &MTX_HANDLE)
                    .unwrap()
            });

        let uid = Sut::unique_id();
        let mut guard = ALLOCATOR_IDS.lock().unwrap();
        assert_that!(!guard.contains(&uid), eq true);
        guard.insert(uid);
    }

    #[conformance_test]
    pub fn growing_and_keep_content_at_front_works<Sut: ShmAllocator>() {
        let chunk_size = CHUNK_SIZE / 8;
        let mut test = Test::<Sut>::new();
        test.init();

        let old_layout = unsafe { Layout::from_size_align_unchecked(chunk_size, 1) };
        let offset = unsafe { test.sut().allocate(old_layout).unwrap() };
        let ptr = test.offset_to_ptr(offset);

        for n in 0..chunk_size {
            unsafe { *ptr.add(n) = n as u8 };
        }

        let new_layout = unsafe { Layout::from_size_align_unchecked(chunk_size * 2, 1) };
        let offset = unsafe {
            test.sut()
                .grow(offset, old_layout, new_layout, ContentPlacement::Front)
                .unwrap()
        };
        let ptr = test.offset_to_ptr(offset);

        for n in 0..chunk_size {
            assert_that!(unsafe {*ptr.add(n)}, eq n as u8);
        }
    }

    #[conformance_test]
    pub fn growing_and_keep_content_at_back_works<Sut: ShmAllocator>() {
        let chunk_size = CHUNK_SIZE / 8;
        let mut test = Test::<Sut>::new();
        test.init();

        let old_layout = unsafe { Layout::from_size_align_unchecked(chunk_size, 1) };
        let offset = unsafe { test.sut().allocate(old_layout).unwrap() };
        let ptr = test.offset_to_ptr(offset);

        for n in 0..chunk_size {
            unsafe { *ptr.add(n) = n as u8 };
        }

        let new_layout = unsafe { Layout::from_size_align_unchecked(chunk_size * 2, 1) };
        let offset = unsafe {
            test.sut()
                .grow(offset, old_layout, new_layout, ContentPlacement::Back)
                .unwrap()
        };
        let ptr = test.offset_to_ptr(offset);

        for n in chunk_size..chunk_size * 2 {
            assert_that!(unsafe { *ptr.add(n) }, eq(n - chunk_size) as u8);
        }
    }

    #[conformance_test]
    pub fn growing_not_last_chunk_and_keep_content_at_front_works<Sut: ShmAllocator>() {
        let chunk_size = CHUNK_SIZE / 8;
        let mut test = Test::<Sut>::new();
        test.init();

        let old_layout = unsafe { Layout::from_size_align_unchecked(chunk_size, 1) };
        let offset = unsafe { test.sut().allocate(old_layout).unwrap() };
        let ptr = test.offset_to_ptr(offset);

        for n in 0..chunk_size {
            unsafe { *ptr.add(n) = n as u8 * 2 };
        }

        let middle_layout = unsafe { Layout::from_size_align_unchecked(128, 1) };
        let middle_chunk = unsafe { test.sut().allocate(middle_layout).unwrap() };
        let ptr = test.offset_to_ptr(middle_chunk);

        for n in 0..middle_layout.size() {
            unsafe { *ptr.add(n) = 91 };
        }

        let new_layout = unsafe { Layout::from_size_align_unchecked(chunk_size * 2, 1) };
        let offset = unsafe {
            test.sut()
                .grow(offset, old_layout, new_layout, ContentPlacement::Front)
                .unwrap()
        };
        let ptr = test.offset_to_ptr(offset);

        for n in 0..chunk_size {
            assert_that!(unsafe {*ptr.add(n)}, eq n as u8 * 2);
        }

        let ptr = test.offset_to_ptr(middle_chunk);
        for n in 0..middle_layout.size() {
            assert_that!(unsafe{*ptr.add(n)}, eq 91);
        }
    }

    #[conformance_test]
    pub fn growing_not_last_chunk_and_keep_content_at_back_works<Sut: ShmAllocator>() {
        let chunk_size = CHUNK_SIZE / 8;
        let mut test = Test::<Sut>::new();
        test.init();

        let old_layout = unsafe { Layout::from_size_align_unchecked(chunk_size, 1) };
        let offset = unsafe { test.sut().allocate(old_layout).unwrap() };
        let ptr = test.offset_to_ptr(offset);

        for n in 0..chunk_size {
            unsafe { *ptr.add(n) = n as u8 };
        }

        let middle_layout = unsafe { Layout::from_size_align_unchecked(128, 1) };
        let middle_chunk = unsafe { test.sut().allocate(middle_layout).unwrap() };
        let ptr = test.offset_to_ptr(middle_chunk);

        for n in 0..middle_layout.size() {
            unsafe { *ptr.add(n) = 47 };
        }

        let new_layout = unsafe { Layout::from_size_align_unchecked(chunk_size * 2, 1) };
        let offset = unsafe {
            test.sut()
                .grow(offset, old_layout, new_layout, ContentPlacement::Back)
                .unwrap()
        };
        let ptr = test.offset_to_ptr(offset);

        for n in chunk_size..chunk_size * 2 {
            assert_that!(unsafe { *ptr.add(n) }, eq(n - chunk_size) as u8);
        }

        let ptr = test.offset_to_ptr(middle_chunk);
        for n in 0..middle_layout.size() {
            assert_that!(unsafe{*ptr.add(n)}, eq 47);
        }
    }

    #[conformance_test]
    pub fn growing_larger_than_memory_is_available_fails<Sut: ShmAllocator>() {
        let chunk_size = CHUNK_SIZE / 8;
        let mut test = Test::<Sut>::new();
        test.init();

        let old_layout = unsafe { Layout::from_size_align_unchecked(chunk_size, 1) };
        let offset = unsafe { test.sut().allocate(old_layout).unwrap() };

        let new_layout = unsafe { Layout::from_size_align_unchecked(MEMORY_SIZE * 2, 1) };
        let offset = unsafe {
            test.sut()
                .grow(offset, old_layout, new_layout, ContentPlacement::Back)
        };

        assert_that!(offset.err(), eq Some(ShmAllocatorGrowError::AllocationGrowError(iceoryx2_bb_memory::pool_allocator::AllocationGrowError::OutOfMemory)));
    }

    #[conformance_test]
    pub fn growing_with_smaller_size_fails<Sut: ShmAllocator>() {
        let chunk_size = CHUNK_SIZE / 8;
        let mut test = Test::<Sut>::new();
        test.init();

        let old_layout = unsafe { Layout::from_size_align_unchecked(chunk_size, 1) };
        let offset = unsafe { test.sut().allocate(old_layout).unwrap() };

        let new_layout = unsafe { Layout::from_size_align_unchecked(chunk_size - 1, 1) };
        let offset = unsafe {
            test.sut()
                .grow(offset, old_layout, new_layout, ContentPlacement::Back)
        };

        assert_that!(offset.err(), eq Some(ShmAllocatorGrowError::AllocationGrowError(iceoryx2_bb_memory::pool_allocator::AllocationGrowError::GrowWouldShrink)));
    }
}
