// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

mod shm_allocator_pool_allocator {
    use core::{alloc::Layout, ptr::NonNull};
    use std::collections::HashSet;

    use iceoryx2_bb_elementary_traits::allocator::AllocationError;
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::{
        shm_allocator::{pool_allocator::*, AllocationStrategy, ShmAllocationError, ShmAllocator},
        zero_copy_connection::PointerOffset,
    };

    const MAX_SUPPORTED_ALIGNMENT: usize = 4096;
    const BUCKET_CONFIG: Layout = unsafe { Layout::from_size_align_unchecked(32, 4) };
    const MEM_SIZE: usize = 16384 * 10;
    const PAYLOAD_SIZE: usize = 8192;

    struct TestContext {
        _payload_memory: Box<[u8; MEM_SIZE]>,
        _base_address: NonNull<[u8]>,
        sut: Box<PoolAllocator>,
    }

    impl TestContext {
        fn new(bucket_layout: Layout) -> Self {
            let mut payload_memory = Box::new([0u8; MEM_SIZE]);
            let base_address =
                unsafe { NonNull::<[u8]>::new_unchecked(&mut payload_memory[0..PAYLOAD_SIZE]) };
            let allocator = BumpAllocator::new(
                unsafe { NonNull::new_unchecked(payload_memory[PAYLOAD_SIZE..].as_mut_ptr()) },
                MEM_SIZE,
            );
            let config = &Config { bucket_layout };
            let mut sut = Box::new(unsafe {
                PoolAllocator::new_uninit(MAX_SUPPORTED_ALIGNMENT, base_address, config)
            });

            unsafe { sut.init(&allocator).unwrap() };

            Self {
                _payload_memory: payload_memory,
                _base_address: base_address,
                sut,
            }
        }
    }

    #[test]
    fn is_setup_correctly() {
        let test_context = TestContext::new(Layout::from_size_align(2, 1).unwrap());

        assert_that!(test_context.sut.number_of_buckets() as usize, eq PAYLOAD_SIZE / 2);
        assert_that!({ test_context.sut.relative_start_address() }, eq 0);

        let test_context = TestContext::new(BUCKET_CONFIG);

        assert_that!(test_context.sut.bucket_size(), eq BUCKET_CONFIG.size());
        assert_that!(test_context.sut.max_alignment(), eq BUCKET_CONFIG.align());
    }

    #[test]
    fn initial_setup_hint_is_layout_times_number_of_chunks() {
        let layout = Layout::from_size_align(64, 2).unwrap();
        let max_number_of_chunks = 54;
        let hint = PoolAllocator::initial_setup_hint(layout, max_number_of_chunks);

        assert_that!(hint.config.bucket_layout, eq layout);
        assert_that!(hint.payload_size, eq layout.size() * max_number_of_chunks);
    }

    fn no_new_resize_hint_when_layout_is_smaller_and_buckets_are_available(
        strategy: AllocationStrategy,
    ) {
        let initial_layout = Layout::from_size_align(12, 4).unwrap();
        let test_context = TestContext::new(initial_layout);
        let hint = test_context
            .sut
            .resize_hint(Layout::from_size_align(8, 2).unwrap(), strategy);

        assert_that!(hint.config.bucket_layout, eq initial_layout);
        assert_that!(hint.payload_size, eq initial_layout.size() * test_context.sut.number_of_buckets() as usize);
    }

    #[test]
    fn no_new_resize_hint_with_power_of_two_when_layout_is_smaller_and_buckets_are_available() {
        no_new_resize_hint_when_layout_is_smaller_and_buckets_are_available(
            AllocationStrategy::PowerOfTwo,
        )
    }

    #[test]
    fn no_new_resize_hint_with_best_fit_when_layout_is_smaller_and_buckets_are_available() {
        no_new_resize_hint_when_layout_is_smaller_and_buckets_are_available(
            AllocationStrategy::BestFit,
        )
    }

    #[test]
    fn new_resize_hint_with_power_of_two_when_layout_is_greater() {
        let initial_layout = Layout::from_size_align(12, 4).unwrap();
        let increased_layout = Layout::from_size_align(28, 2).unwrap();
        let test_context = TestContext::new(initial_layout);
        let hint = test_context
            .sut
            .resize_hint(increased_layout, AllocationStrategy::PowerOfTwo);
        assert_that!(hint.config.bucket_layout.size(), eq increased_layout.size().next_power_of_two());
        assert_that!(hint.config.bucket_layout.align(), eq initial_layout.align());
        assert_that!(hint.payload_size, eq increased_layout.size().next_power_of_two() * test_context.sut.number_of_buckets() as usize);
    }

    #[test]
    fn new_resize_hint_with_best_fit_when_layout_is_greater() {
        let initial_layout = Layout::from_size_align(12, 4).unwrap();
        let increased_layout = Layout::from_size_align(28, 2).unwrap();
        let test_context = TestContext::new(initial_layout);
        let hint = test_context
            .sut
            .resize_hint(increased_layout, AllocationStrategy::BestFit);
        assert_that!(hint.config.bucket_layout.size(), eq increased_layout.size());
        assert_that!(hint.config.bucket_layout.align(), eq initial_layout.align());
        assert_that!(hint.payload_size, eq increased_layout.size() * test_context.sut.number_of_buckets() as usize);
    }

    #[test]
    fn new_resize_hint_with_power_of_two_when_buckets_are_exhausted() {
        let initial_layout = Layout::from_size_align(12, 4).unwrap();
        let increased_layout = Layout::from_size_align(14, 8).unwrap();
        let test_context = TestContext::new(initial_layout);

        for _ in 0..test_context.sut.number_of_buckets() {
            assert_that!(unsafe { test_context.sut.allocate(initial_layout) }, is_ok);
        }

        assert_that!(
            unsafe { test_context.sut.allocate(increased_layout) },
            is_err
        );

        let hint = test_context
            .sut
            .resize_hint(increased_layout, AllocationStrategy::PowerOfTwo);
        assert_that!(hint.config.bucket_layout.size(), eq increased_layout.size().next_power_of_two());
        assert_that!(hint.config.bucket_layout.align(), eq increased_layout.align());
        assert_that!(hint.payload_size, eq increased_layout.size().next_power_of_two() * (test_context.sut.number_of_buckets() + 1).next_power_of_two() as usize);
    }

    #[test]
    fn new_resize_hint_with_best_fit_when_buckets_are_exhausted() {
        let initial_layout = Layout::from_size_align(12, 4).unwrap();
        let increased_layout = Layout::from_size_align(16, 8).unwrap();
        let test_context = TestContext::new(initial_layout);

        for _ in 0..test_context.sut.number_of_buckets() {
            assert_that!(unsafe { test_context.sut.allocate(initial_layout) }, is_ok);
        }

        assert_that!(
            unsafe { test_context.sut.allocate(increased_layout) },
            is_err
        );

        let hint = test_context
            .sut
            .resize_hint(increased_layout, AllocationStrategy::BestFit);
        assert_that!(hint.config.bucket_layout.size(), eq increased_layout.size());
        assert_that!(hint.config.bucket_layout.align(), eq increased_layout.align());
        assert_that!(hint.payload_size, eq increased_layout.size() * (test_context.sut.number_of_buckets() + 1) as usize);
    }

    #[test]
    fn allocate_and_release_all_buckets_works() {
        const REPETITIONS: usize = 10;
        let test_context = TestContext::new(BUCKET_CONFIG);

        for _ in 0..REPETITIONS {
            let mut mem_set = HashSet::new();
            for _ in 0..test_context.sut.number_of_buckets() {
                let memory = unsafe { test_context.sut.allocate(BUCKET_CONFIG).unwrap() };
                // the returned offset must be a multiple of the bucket size
                assert_that!((memory.offset() - test_context.sut.relative_start_address()) % BUCKET_CONFIG.size(), eq 0);
                assert_that!(mem_set.insert(memory.offset()), eq true);
            }

            assert_that!(unsafe { test_context.sut.allocate(BUCKET_CONFIG) }, eq Err(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)));

            for memory in mem_set {
                unsafe {
                    test_context
                        .sut
                        .deallocate(PointerOffset::new(memory), BUCKET_CONFIG)
                }
            }
        }
    }

    #[test]
    fn allocate_twice_release_once_until_memory_is_exhausted_works() {
        const REPETITIONS: usize = 10;
        let test_context = TestContext::new(BUCKET_CONFIG);

        for _ in 0..REPETITIONS {
            let mut mem_set = HashSet::new();
            for _ in 0..(test_context.sut.number_of_buckets() - 1) {
                let memory_1 = unsafe { test_context.sut.allocate(BUCKET_CONFIG).unwrap() };
                // the returned offset must be a multiple of the bucket size
                assert_that!((memory_1.offset() - test_context.sut.relative_start_address()) % BUCKET_CONFIG.size(), eq 0);

                let memory_2 = unsafe { test_context.sut.allocate(BUCKET_CONFIG).unwrap() };
                // the returned offset must be a multiple of the bucket size
                assert_that!((memory_2.offset() - test_context.sut.relative_start_address()) % BUCKET_CONFIG.size(), eq 0);
                assert_that!(mem_set.insert(memory_2.offset()), eq true);

                unsafe {
                    test_context.sut.deallocate(memory_1, BUCKET_CONFIG);
                }
            }

            let memory = unsafe { test_context.sut.allocate(BUCKET_CONFIG).unwrap() };
            // the returned offset must be a multiple of the bucket size
            assert_that!((memory.offset() - test_context.sut.relative_start_address()) % BUCKET_CONFIG.size(), eq 0);
            assert_that!(mem_set.insert(memory.offset()), eq true);

            assert_that!(unsafe { test_context.sut.allocate(BUCKET_CONFIG) }, eq Err(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)));

            for memory in mem_set {
                unsafe {
                    test_context
                        .sut
                        .deallocate(PointerOffset::new(memory), BUCKET_CONFIG)
                }
            }
        }
    }

    #[test]
    fn allocated_memory_has_correct_alignment_uniform_alignment_case() {
        for i in 0..12 {
            for n in 0..=i {
                let layout = Layout::from_size_align(2_usize.pow(i), 2_usize.pow(i)).unwrap();
                let test_context = TestContext::new(layout);

                let mem_layout =
                    Layout::from_size_align(128.min(2_usize.pow(i)), 2_usize.pow(n)).unwrap();
                let mut counter = 0;
                while let Ok(memory) = unsafe { test_context.sut.allocate(mem_layout) } {
                    assert_that!(memory.offset() % mem_layout.align(), eq 0 );
                    counter += 1;
                }

                // just to make sure that actually samples are allocated
                assert_that!(counter, ge 1);
            }
        }
    }

    #[test]
    fn allocated_memory_has_correct_alignment_mixed_alignment_case() {
        for i in 0..12 {
            let layout = Layout::from_size_align(2_usize.pow(i), 2_usize.pow(i)).unwrap();
            let test_context = TestContext::new(layout);

            let mut counter = 0;
            let mut keep_running = true;
            while keep_running {
                for n in 0..=i {
                    let mem_layout =
                        Layout::from_size_align(128.min(2_usize.pow(i)), 2_usize.pow(n)).unwrap();

                    if let Ok(memory) = unsafe { test_context.sut.allocate(mem_layout) } {
                        assert_that!(memory.offset() % mem_layout.align(), eq 0 );
                        counter += 1;
                    } else {
                        keep_running = false;
                        break;
                    }
                }
            }

            // just to make sure that actually samples are allocated
            assert_that!(counter, ge 2);
        }
    }

    #[test]
    fn allocate_with_unsupported_alignment_fails() {
        let test_context =
            TestContext::new(Layout::from_size_align(BUCKET_CONFIG.size(), 1).unwrap());
        assert_that!(unsafe { test_context.sut.allocate(BUCKET_CONFIG) }, eq Err(ShmAllocationError::ExceedsMaxSupportedAlignment));
    }
}
