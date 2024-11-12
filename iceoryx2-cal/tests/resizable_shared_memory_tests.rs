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
    use iceoryx2_cal::testing::*;
    use iceoryx2_cal::{
        shared_memory::SharedMemory,
        shm_allocator::{pool_allocator::PoolAllocator, ShmAllocator},
    };

    type DefaultAllocator = PoolAllocator;
    type AllocatorConfig = <DefaultAllocator as ShmAllocator>::Configuration;

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
            .max_number_of_chunks_hint(1)
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
            .max_number_of_chunks_hint(1)
            .open()
            .unwrap();

        let ptr_creator_1 = sut_creator.allocate(Layout::new::<u64>()).unwrap();
        let ptr_creator_2 = sut_creator.allocate(Layout::new::<u64>()).unwrap();

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

    #[instantiate_tests(<iceoryx2_cal::shared_memory::posix::Memory<DefaultAllocator>, resizable_shared_memory::dynamic::DynamicMemory<DefaultAllocator, iceoryx2_cal::shared_memory::posix::Memory<DefaultAllocator>>>)]
    mod posix {}

    #[instantiate_tests(<iceoryx2_cal::shared_memory::process_local::Memory<DefaultAllocator>, resizable_shared_memory::dynamic::DynamicMemory<DefaultAllocator, iceoryx2_cal::shared_memory::process_local::Memory<DefaultAllocator>>>)]
    mod process_local {}
}
