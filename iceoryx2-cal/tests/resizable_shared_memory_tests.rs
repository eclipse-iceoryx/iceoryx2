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

#[generic_tests::define]
mod resizable_shared_memory {
    use std::alloc::Layout;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::resizable_shared_memory::{self, *};
    use iceoryx2_cal::shm_allocator::{AllocationError, AllocationStrategy, ShmAllocationError};
    use iceoryx2_cal::testing::*;
    use iceoryx2_cal::{shared_memory::SharedMemory, shm_allocator::pool_allocator::PoolAllocator};

    type DefaultAllocator = PoolAllocator;

    #[test]
    fn create_and_open_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_creator = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_number_of_chunks_hint(1)
            .create()
            .unwrap();
        let mut sut_viewer = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

        let test_value_1 = 189273771;
        let test_value_2 = 90912638975;
        let ptr_creator = sut_creator.allocate(Layout::new::<u64>()).unwrap();

        unsafe { (ptr_creator.data_ptr as *mut u64).write(test_value_1) };

        let ptr_view = sut_viewer
            .register_and_translate_offset(ptr_creator.offset)
            .unwrap() as *const u64;

        assert_that!(unsafe{ *ptr_view }, eq test_value_1);
        unsafe { (ptr_creator.data_ptr as *mut u64).write(test_value_2) };
        assert_that!(unsafe{ *ptr_view }, eq test_value_2);
    }

    #[test]
    fn allocate_more_layout_than_hinted_when_no_other_chunks_are_in_use_releases_smaller_segment<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(128)
            .create()
            .unwrap();

        sut.allocate(Layout::new::<u16>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 1);
        sut.allocate(Layout::new::<u32>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 2);
        sut.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 3);
    }

    #[test]
    fn allocate_more_layout_than_hinted_when_other_chunks_are_in_use_does_not_releases_smaller_segment<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(128)
            .create()
            .unwrap();

        sut.allocate(Layout::new::<u8>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 1);
        sut.allocate(Layout::new::<u16>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 2);
        sut.allocate(Layout::new::<u32>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 3);
        sut.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut.number_of_active_segments(), eq 4);
    }

    #[test]
    fn allocate_more_than_hinted_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_creator = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_number_of_chunks_hint(1)
            .create()
            .unwrap();
        let mut sut_viewer = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

        let ptr_creator_1 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut_creator.number_of_active_segments(), eq 1);
        let ptr_creator_2 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut_creator.number_of_active_segments(), eq 2);

        let test_value_1 = 109875896345234897;
        let test_value_2 = 412384034975234569;

        unsafe { (ptr_creator_1.data_ptr as *mut u64).write(test_value_1) };
        unsafe { (ptr_creator_2.data_ptr as *mut u64).write(test_value_2) };

        let ptr_view_1 = sut_viewer
            .register_and_translate_offset(ptr_creator_1.offset)
            .unwrap() as *const u64;
        let ptr_view_2 = sut_viewer
            .register_and_translate_offset(ptr_creator_2.offset)
            .unwrap() as *const u64;

        assert_that!(unsafe{ *ptr_view_1 }, eq test_value_1);
        assert_that!(unsafe{ *ptr_view_2 }, eq test_value_2);
    }

    #[test]
    fn deallocate_removes_unused_segments_on_creator_side<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_creator = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_number_of_chunks_hint(1)
            .create()
            .unwrap();

        let ptr_creator_1 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut_creator.number_of_active_segments(), eq 1);

        let _ptr_creator_2 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut_creator.number_of_active_segments(), eq 2);

        unsafe { sut_creator.deallocate(ptr_creator_1.offset, Layout::new::<u64>()) };
        assert_that!(sut_creator.number_of_active_segments(), eq 1);
    }

    #[test]
    fn unregister_removes_unused_segments_on_viewer_side<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_creator = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_number_of_chunks_hint(1)
            .create()
            .unwrap();

        let ptr_creator_1 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut_creator.number_of_active_segments(), eq 1);

        let _ptr_creator_2 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        assert_that!(sut_creator.number_of_active_segments(), eq 2);

        unsafe { sut_creator.deallocate(ptr_creator_1.offset, Layout::new::<u64>()) };
        assert_that!(sut_creator.number_of_active_segments(), eq 1);
    }

    fn allocate_more_than_hinted_with_increasing_chunk_size_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >(
        strategy: AllocationStrategy,
    ) {
        const NUMBER_OF_REALLOCATIONS: usize = 128;
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_creator = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(1)
            .allocation_strategy(strategy)
            .create()
            .unwrap();

        let mut sut_viewer = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

        let mut ptrs = vec![];
        for i in 0..NUMBER_OF_REALLOCATIONS {
            let size = 2 + i;
            let layout = unsafe { Layout::from_size_align_unchecked(size, 1) };
            let ptr = sut_creator.allocate(layout).unwrap();

            for n in 0..size {
                unsafe { ptr.data_ptr.add(n).write(i as u8) };
            }
            ptrs.push(ptr);
        }

        for i in 0..NUMBER_OF_REALLOCATIONS {
            let size = 2 + i;
            let ptr_view = sut_viewer
                .register_and_translate_offset(ptrs[i].offset)
                .unwrap();

            for n in 0..size {
                assert_that!(unsafe{ *ptr_view.add(n) }, eq i as u8);
            }
        }
    }

    #[test]
    fn allocate_more_than_hinted_with_increasing_chunk_size_and_best_fit_strategy_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        allocate_more_than_hinted_with_increasing_chunk_size_works::<Shm, Sut>(
            AllocationStrategy::BestFit,
        );
    }

    #[test]
    fn allocate_more_than_hinted_with_increasing_chunk_size_and_power_of_two_strategy_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        allocate_more_than_hinted_with_increasing_chunk_size_works::<Shm, Sut>(
            AllocationStrategy::PowerOfTwo,
        );
    }

    fn allocate_with_sufficient_chunk_hint_and_increasing_size<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >(
        strategy: AllocationStrategy,
    ) {
        const NUMBER_OF_REALLOCATIONS: usize = 128;
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_creator = Sut::Builder::new(&storage_name)
            .config(&config)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(NUMBER_OF_REALLOCATIONS)
            .allocation_strategy(strategy)
            .create()
            .unwrap();

        let mut sut_viewer = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

        let mut ptrs = vec![];
        for i in 0..NUMBER_OF_REALLOCATIONS {
            let size = 2 + i;
            let layout = unsafe { Layout::from_size_align_unchecked(size, 1) };
            let ptr = sut_creator.allocate(layout).unwrap();

            for n in 0..size {
                unsafe { ptr.data_ptr.add(n).write(2 * i as u8) };
            }
            ptrs.push(ptr);
        }

        for i in 0..NUMBER_OF_REALLOCATIONS {
            let size = 2 + i;
            let ptr_view = sut_viewer
                .register_and_translate_offset(ptrs[i].offset)
                .unwrap();

            for n in 0..size {
                assert_that!(unsafe{ *ptr_view.add(n) }, eq 2*i as u8);
            }
        }
    }

    #[test]
    fn allocate_with_sufficient_chunk_hint_and_increasing_size_strategy_power_of_two<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        allocate_with_sufficient_chunk_hint_and_increasing_size::<Shm, Sut>(
            AllocationStrategy::PowerOfTwo,
        )
    }

    #[test]
    fn allocate_with_sufficient_chunk_hint_and_increasing_size_strategy_best_fit<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        allocate_with_sufficient_chunk_hint_and_increasing_size::<Shm, Sut>(
            AllocationStrategy::BestFit,
        )
    }

    #[test]
    fn deallocate_last_segment_does_not_release_it<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .allocation_strategy(AllocationStrategy::Static)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(1)
            .create()
            .unwrap();

        assert_that!(sut.allocate(Layout::new::<u8>()), is_ok);

        let result = sut.allocate(Layout::new::<u8>());
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq ResizableShmAllocationError::ShmAllocationError(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)));
    }

    #[test]
    fn static_allocation_strategy_does_not_resize_available_chunks<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .allocation_strategy(AllocationStrategy::Static)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(8)
            .create()
            .unwrap();

        let result = sut.allocate(Layout::from_size_align(8, 1).unwrap());
        assert_that!(result, is_err);
    }

    #[test]
    fn static_allocation_strategy_increase_available_chunks<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .allocation_strategy(AllocationStrategy::Static)
            .max_chunk_layout_hint(Layout::new::<u8>())
            .max_number_of_chunks_hint(1)
            .create()
            .unwrap();

        let result = sut.allocate(Layout::new::<u8>());
        assert_that!(result, is_ok);
        let result = sut.allocate(Layout::new::<u8>());
        assert_that!(result, is_err);
    }

    #[test]
    fn list_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        const NUMBER_OF_STORAGES: usize = 28;
        let config = generate_isolated_config::<Sut>();

        let mut suts = vec![];
        let mut names = vec![];

        for _ in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            let sut = Sut::Builder::new(&storage_name)
                .config(&config)
                .create()
                .unwrap();
            names.push(storage_name);
            suts.push(sut);
        }

        let list_suts = Sut::list_cfg(&config).unwrap();
        assert_that!(list_suts, len names.len());
        for name in names {
            assert_that!(list_suts, contains name);
        }
    }

    #[test]
    fn list_works_when_the_start_segment_is_no_longer_used<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        const NUMBER_OF_STORAGES: usize = 33;
        let config = generate_isolated_config::<Sut>();

        let mut suts = vec![];
        let mut names = vec![];

        for _ in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            let sut = Sut::Builder::new(&storage_name)
                .config(&config)
                .max_chunk_layout_hint(Layout::new::<u8>())
                .allocation_strategy(AllocationStrategy::BestFit)
                .create()
                .unwrap();

            // this allocates a new segment and release the original one
            sut.allocate(Layout::new::<u16>()).unwrap();
            assert_that!(sut.number_of_active_segments(), eq 1);

            // this allocates a new segment
            sut.allocate(Layout::new::<u64>()).unwrap();
            assert_that!(sut.number_of_active_segments(), eq 2);

            names.push(storage_name);
            suts.push(sut);
        }

        let list_suts = Sut::list_cfg(&config).unwrap();
        assert_that!(list_suts, len names.len());
        for name in names {
            assert_that!(list_suts, contains name);
        }
    }

    #[test]
    fn does_exist_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        const NUMBER_OF_STORAGES: usize = 25;
        let config = generate_isolated_config::<Sut>();

        let mut suts = vec![];
        let mut names = vec![];

        for _ in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            let sut = Sut::Builder::new(&storage_name)
                .config(&config)
                .create()
                .unwrap();
            names.push(storage_name);
            suts.push(sut);
        }

        for name in names {
            assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        }
    }

    #[test]
    fn does_exist_works_when_the_start_segment_is_no_longer_used<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        const NUMBER_OF_STORAGES: usize = 25;
        let config = generate_isolated_config::<Sut>();

        let mut suts = vec![];
        let mut names = vec![];

        for _ in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            let sut = Sut::Builder::new(&storage_name)
                .config(&config)
                .max_chunk_layout_hint(Layout::new::<u8>())
                .allocation_strategy(AllocationStrategy::BestFit)
                .create()
                .unwrap();

            // this allocates a new segment and release the original one
            sut.allocate(Layout::new::<u16>()).unwrap();
            assert_that!(sut.number_of_active_segments(), eq 1);

            // this allocates a new segment
            sut.allocate(Layout::new::<u64>()).unwrap();
            assert_that!(sut.number_of_active_segments(), eq 2);

            names.push(storage_name);
            suts.push(sut);
        }

        for name in names {
            assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        }
    }

    #[test]
    fn remove_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        const NUMBER_OF_STORAGES: usize = 26;
        let config = generate_isolated_config::<Sut>();

        let mut names = vec![];

        for _ in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(false));
            let sut = Sut::Builder::new(&storage_name)
                .config(&config)
                .create()
                .unwrap();
            core::mem::forget(sut);
            names.push(storage_name);
        }

        for name in names {
            assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
            assert_that!(unsafe { Sut::remove_cfg(&name, &config) }, eq Ok(true));
            assert_that!(unsafe { Sut::remove_cfg(&name, &config) }, eq Ok(false));
            assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
        }
    }

    #[test]
    fn remove_with_multiple_segments_works<
        Shm: SharedMemory<DefaultAllocator>,
        Sut: ResizableSharedMemory<DefaultAllocator, Shm>,
    >() {
        const NUMBER_OF_STORAGES: usize = 26;
        let config = generate_isolated_config::<Sut>();

        let mut names = vec![];

        for _ in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(false));
            let sut = Sut::Builder::new(&storage_name)
                .config(&config)
                .max_chunk_layout_hint(Layout::new::<u8>())
                .max_number_of_chunks_hint(123)
                .allocation_strategy(AllocationStrategy::BestFit)
                .create()
                .unwrap();

            sut.allocate(Layout::new::<u8>()).unwrap();
            sut.allocate(Layout::new::<u16>()).unwrap();
            sut.allocate(Layout::new::<u32>()).unwrap();
            sut.allocate(Layout::new::<u64>()).unwrap();
            assert_that!(sut.number_of_active_segments(), eq 4);

            core::mem::forget(sut);
            names.push(storage_name);
        }

        for name in names {
            assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
            assert_that!(unsafe { Sut::remove_cfg(&name, &config) }, eq Ok(true));
            assert_that!(unsafe { Sut::remove_cfg(&name, &config) }, eq Ok(false));
            assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
        }
    }

    // TODO:
    //  * open with no more __0 segment
    //  * open with many segments
    //  * AllocationStrategy::PowerOfTwo -> doubling in size
    //  * has_ownership, acquire/release ownership
    //  * timeout
    //  * best fit, let reallocate until 256 exceeded, see if id is recycled
    //  * exceed max alignment
    //  * separate builder for view, without hints
    //  * start with layout.size == 1 and max_number_of_chunks == 1
    //    * allocate 1 byte
    //    * allocate N byte, may lead to 2 allocations, one for chunk resize, one for bucket number
    //      resize

    #[instantiate_tests(<iceoryx2_cal::shared_memory::posix::Memory<DefaultAllocator>, resizable_shared_memory::dynamic::DynamicMemory<DefaultAllocator, iceoryx2_cal::shared_memory::posix::Memory<DefaultAllocator>>>)]
    mod posix {}

    #[instantiate_tests(<iceoryx2_cal::shared_memory::process_local::Memory<DefaultAllocator>, resizable_shared_memory::dynamic::DynamicMemory<DefaultAllocator, iceoryx2_cal::shared_memory::process_local::Memory<DefaultAllocator>>>)]
    mod process_local {}
}
