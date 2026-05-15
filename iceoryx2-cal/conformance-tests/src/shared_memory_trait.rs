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
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;

pub type DefaultAllocator = PoolAllocator;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod shared_memory_trait {
    use alloc::vec;
    use core::alloc::Layout;
    use iceoryx2_bb_posix::file::AccessMode;
    use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY;

    use iceoryx2_bb_posix::testing::generate_file_path;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::shared_memory::*;
    use iceoryx2_cal::shm_allocator::{ShmAllocationError, ShmAllocator};
    use iceoryx2_cal::testing::*;

    use super::*;

    type AllocatorConfig = <DefaultAllocator as ShmAllocator>::Configuration;

    const NUMBER_OF_CHUNKS: usize = 32;
    const CHUNK_SIZE: usize = 8192;
    const DEFAULT_SIZE: usize = CHUNK_SIZE * NUMBER_OF_CHUNKS;
    const DEFAULT_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
    const SHM_CONFIG: AllocatorConfig = AllocatorConfig {
        bucket_layout: DEFAULT_LAYOUT,
    };

    #[conformance_test]
    pub fn size_of_zero_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create = Sut::Builder::new(&name)
            .size(0)
            .config(&config)
            .create(&SHM_CONFIG);
        let sut_open = Sut::Builder::new(&name)
            .config(&config)
            .open(AccessMode::ReadWrite);

        assert_that!(sut_create, is_err);
        assert_that!(sut_open, is_err);

        assert_that!(
            sut_create.err().unwrap(), eq
            SharedMemoryCreateError::SizeIsZero
        );
        assert_that!(sut_open.err().unwrap(), eq SharedMemoryOpenError::DoesNotExist);
    }

    #[conformance_test]
    pub fn non_zero_size_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();
        let sut_open = Sut::Builder::new(&name)
            .config(&config)
            .open(AccessMode::ReadWrite)
            .unwrap();

        assert_that!(sut_create.size(), ge DEFAULT_SIZE);
        assert_that!(sut_open.size(), ge DEFAULT_SIZE);
    }

    #[conformance_test]
    pub fn create_after_drop_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG);

        assert_that!(sut_create, is_ok);
        drop(sut_create);

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG);

        assert_that!(sut_create, is_ok);
    }

    #[conformance_test]
    pub fn creating_it_twice_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create1 = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG);
        let sut_create2 = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG);

        assert_that!(sut_create1, is_ok);
        assert_that!(sut_create2, is_err);

        assert_that!(
            sut_create2.err().unwrap(), eq
            SharedMemoryCreateError::AlreadyExists
        );
    }

    #[conformance_test]
    pub fn opening_non_existing_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_open = Sut::Builder::new(&name)
            .config(&config)
            .open(AccessMode::ReadWrite);

        assert_that!(sut_open, is_err);

        assert_that!(sut_open.err().unwrap(), eq SharedMemoryOpenError::DoesNotExist);
    }

    #[conformance_test]
    pub fn allocation_with_creator_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();
        let mut chunks = vec![];

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();

        for _ in 0..NUMBER_OF_CHUNKS {
            let chunk = sut_create.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let chunk = sut_create.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_err);
        assert_that!(
            chunk.err().unwrap(), eq
            ShmAllocationError::AllocationError(AllocationError::OutOfMemory)
        );

        unsafe {
            sut_create.deallocate(chunks[0].offset, DEFAULT_LAYOUT);
        }

        let chunk = sut_create.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_ok);
    }

    #[conformance_test]
    pub fn allocation_with_creator_and_client_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();
        let mut chunks = vec![];

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();

        for _ in 0..NUMBER_OF_CHUNKS / 2 {
            let chunk = sut_create.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let sut_open = Sut::Builder::new(&name)
            .config(&config)
            .open(AccessMode::ReadWrite)
            .unwrap();
        for _ in 0..NUMBER_OF_CHUNKS / 2 {
            let chunk = sut_open.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let chunk = sut_open.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_err);
        assert_that!(
            chunk.err().unwrap(), eq
            ShmAllocationError::AllocationError(AllocationError::OutOfMemory)
        );

        unsafe {
            sut_create.deallocate(chunks[0].offset, DEFAULT_LAYOUT);
        }

        let chunk = sut_open.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_ok);
    }

    #[conformance_test]
    pub fn allocated_chunks_have_correct_alignment<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        for i in 0..12 {
            let layout = unsafe { Layout::from_size_align_unchecked(128, 2_usize.pow(i)) };
            let shm_config = AllocatorConfig {
                bucket_layout: layout,
            };

            for n in 0..=i {
                let sut_create = Sut::Builder::new(&name)
                    .size(DEFAULT_SIZE)
                    .config(&config)
                    .create(&shm_config)
                    .unwrap();

                let chunk_layout =
                    unsafe { Layout::from_size_align_unchecked(2_usize.pow(n), 2_usize.pow(n)) };

                while let Ok(chunk) = sut_create.allocate(chunk_layout) {
                    assert_that!((chunk.data_ptr as usize) % chunk_layout.align(), eq 0);
                }
            }
        }
    }

    #[conformance_test]
    pub fn defaults_for_configuration_are_set_correctly<Sut: SharedMemory<DefaultAllocator>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
    }

    #[conformance_test]
    pub fn shm_does_not_remove_resources_without_ownership<Sut: SharedMemory<DefaultAllocator>>() {
        test_requires!(Sut::does_support_persistency());
        let config = generate_isolated_config::<Sut>();

        let name_1 = generate_file_path().file_name();
        let name_2 = generate_file_path().file_name();

        let sut_1 = Sut::Builder::new(&name_1)
            .size(1024)
            .has_ownership(true)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();
        sut_1.release_ownership();

        let sut_2 = Sut::Builder::new(&name_2)
            .size(1024)
            .has_ownership(false)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();

        assert_that!(sut_1.has_ownership(), eq false);
        assert_that!(sut_2.has_ownership(), eq false);

        drop(sut_1);
        drop(sut_2);

        assert_that!(Sut::does_exist_cfg(&name_1, &config), eq Ok(true));
        assert_that!(Sut::does_exist_cfg(&name_2, &config), eq Ok(true));

        assert_that!(unsafe { Sut::remove_cfg(&name_1, &config) }, eq Ok(true));
        assert_that!(unsafe { Sut::remove_cfg(&name_2, &config) }, eq Ok(true));
    }

    #[conformance_test]
    pub fn acquired_ownership_leads_to_cleaned_up_resources<Sut: SharedMemory<DefaultAllocator>>() {
        test_requires!(Sut::does_support_persistency());

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&name)
            .size(1024)
            .has_ownership(false)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();

        assert_that!(sut.has_ownership(), eq false);
        sut.acquire_ownership();
        assert_that!(sut.has_ownership(), eq true);

        drop(sut);
        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(false));
    }

    #[conformance_test]
    pub fn abandoning_keeps_resources_alive<Sut: SharedMemory<DefaultAllocator>>() {
        test_requires!(POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY);

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .has_ownership(true)
            .create(&SHM_CONFIG)
            .unwrap();

        sut_create.abandon();

        assert_that!(unsafe { Sut::remove_cfg(&name, &config) }, eq Ok(true));
    }
}
