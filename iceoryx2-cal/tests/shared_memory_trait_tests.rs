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
mod shared_memory {
    use core::alloc::Layout;

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;
    use iceoryx2_cal::{
        named_concept::NamedConceptBuilder,
        shared_memory::*,
        shm_allocator::{pool_allocator::PoolAllocator, ShmAllocationError, ShmAllocator},
    };

    type DefaultAllocator = PoolAllocator;
    type AllocatorConfig = <DefaultAllocator as ShmAllocator>::Configuration;

    const NUMBER_OF_CHUNKS: usize = 32;
    const CHUNK_SIZE: usize = 8192;
    const DEFAULT_SIZE: usize = CHUNK_SIZE * NUMBER_OF_CHUNKS;
    const DEFAULT_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
    const SHM_CONFIG: AllocatorConfig = AllocatorConfig {
        bucket_layout: DEFAULT_LAYOUT,
    };

    #[test]
    fn size_of_zero_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create = Sut::Builder::new(&name)
            .size(0)
            .config(&config)
            .create(&SHM_CONFIG);
        let sut_open = Sut::Builder::new(&name).config(&config).open();

        assert_that!(sut_create, is_err);
        assert_that!(sut_open, is_err);

        assert_that!(
            sut_create.err().unwrap(), eq
            SharedMemoryCreateError::SizeIsZero
        );
        assert_that!(sut_open.err().unwrap(), eq SharedMemoryOpenError::DoesNotExist);
    }

    #[test]
    fn non_zero_size_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .config(&config)
            .create(&SHM_CONFIG)
            .unwrap();
        let sut_open = Sut::Builder::new(&name).config(&config).open().unwrap();

        assert_that!(sut_create.size(), ge DEFAULT_SIZE);
        assert_that!(sut_open.size(), ge DEFAULT_SIZE);
    }

    #[test]
    fn create_after_drop_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
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

    #[test]
    fn creating_it_twice_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
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

    #[test]
    fn opening_non_existing_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_open = Sut::Builder::new(&name).config(&config).open();

        assert_that!(sut_open, is_err);

        assert_that!(sut_open.err().unwrap(), eq SharedMemoryOpenError::DoesNotExist);
    }

    #[test]
    fn allocation_with_creator_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
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

    #[test]
    fn allocation_with_creator_and_client_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
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

        let sut_open = Sut::Builder::new(&name).config(&config).open().unwrap();
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

    #[test]
    fn allocated_chunks_have_correct_alignment<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
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

    #[test]
    fn list_shm_works<Sut: SharedMemory<DefaultAllocator>>() {
        let mut storage_names = vec![];
        let mut storages = vec![];
        const LIMIT: usize = 8;
        let config = generate_isolated_config::<Sut>();

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len i);
            storage_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_names[i], &config), eq Ok(false));
            storages.push(
                Sut::Builder::new(&storage_names[i])
                    .size(DEFAULT_SIZE)
                    .config(&config)
                    .create(&SHM_CONFIG)
                    .unwrap(),
            );
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_names[i], &config), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list_cfg(&config).unwrap();
            assert_that!(list, len i + 1);
            let does_exist_in_list = |value| {
                for e in &list {
                    if e == value {
                        return true;
                    }
                }
                false
            };

            for name in &storage_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len LIMIT);

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&storage_names[i], &config)}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&storage_names[i], &config)}, eq Ok(false));
        }

        core::mem::forget(storages);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_storages_separated<Sut: SharedMemory<DefaultAllocator>>() {
        let config = generate_isolated_config::<Sut>();
        let config_1 = config
            .clone()
            .suffix(unsafe { &FileName::new_unchecked(b".s1") });
        let config_2 = config.suffix(unsafe { &FileName::new_unchecked(b".s2") });

        let storage_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let storage_guard_1 = Sut::Builder::new(&storage_name)
            .size(DEFAULT_SIZE)
            .config(&config_1)
            .create(&SHM_CONFIG)
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let storage_guard_2 = Sut::Builder::new(&storage_name)
            .size(DEFAULT_SIZE)
            .config(&config_2)
            .create(&SHM_CONFIG)
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq storage_name);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq storage_name);

        assert_that!(*storage_guard_1.name(), eq storage_name);
        assert_that!(*storage_guard_2.name(), eq storage_name);

        storage_guard_1.release_ownership();
        storage_guard_2.release_ownership();

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: SharedMemory<DefaultAllocator>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
        assert_that!(*config.get_path_hint(), eq Sut::default_path_hint());
        assert_that!(*config.get_prefix(), eq Sut::default_prefix());
    }

    #[test]
    fn shm_does_not_remove_resources_without_ownership<Sut: SharedMemory<DefaultAllocator>>() {
        test_requires!(Sut::does_support_persistency());
        let config = generate_isolated_config::<Sut>();

        let name_1 = generate_name();
        let name_2 = generate_name();

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

    #[test]
    fn acquired_ownership_leads_to_cleaned_up_resources<Sut: SharedMemory<DefaultAllocator>>() {
        test_requires!(Sut::does_support_persistency());

        let name = generate_name();
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

    #[instantiate_tests(<iceoryx2_cal::shared_memory::posix::Memory<DefaultAllocator>>)]
    mod posix {}

    #[instantiate_tests(<iceoryx2_cal::shared_memory::process_local::Memory<DefaultAllocator>>)]
    mod process_local {}
}
