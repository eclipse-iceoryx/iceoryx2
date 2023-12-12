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
    use std::alloc::Layout;

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_elementary::math::ToB64;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::{
        named_concept::NamedConceptBuilder,
        shared_memory::*,
        shm_allocator::{pool_allocator::PoolAllocator, ShmAllocationError, ShmAllocator},
    };

    type DefaultAllocator = PoolAllocator;
    type AllocatorConfig = <DefaultAllocator as ShmAllocator>::Configuration;

    const NUMBER_OF_CHUNKS: usize = 16;
    const CHUNK_SIZE: usize = 128;
    const DEFAULT_SIZE: usize = CHUNK_SIZE * NUMBER_OF_CHUNKS;
    const DEFAULT_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(CHUNK_SIZE, 1) };
    const SHM_CONFIG: AllocatorConfig = AllocatorConfig {
        bucket_layout: DEFAULT_LAYOUT,
    };

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"test_").unwrap();
        file.push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
            .unwrap();
        file
    }

    #[test]
    fn size_of_zero_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();

        let sut_create = Sut::Builder::new(&name).size(0).create(&SHM_CONFIG);
        let sut_open = Sut::Builder::new(&name).open();

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

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .create(&SHM_CONFIG)
            .unwrap();
        let sut_open = Sut::Builder::new(&name).open().unwrap();

        assert_that!(sut_create.size(), ge DEFAULT_SIZE);
        assert_that!(sut_open.size(), ge DEFAULT_SIZE);
    }

    #[test]
    fn create_after_drop_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .create(&SHM_CONFIG);

        assert_that!(sut_create, is_ok);
        drop(sut_create);

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .create(&SHM_CONFIG);

        assert_that!(sut_create, is_ok);
    }

    #[test]
    fn creating_it_twice_fails<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();

        let sut_create1 = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .create(&SHM_CONFIG);
        let sut_create2 = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
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

        let sut_open = Sut::Builder::new(&name).open();

        assert_that!(sut_open, is_err);

        assert_that!(sut_open.err().unwrap(), eq SharedMemoryOpenError::DoesNotExist);
    }

    #[test]
    fn allocation_with_creator_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
        let mut chunks = vec![];

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
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
            assert_that!(
                sut_create.deallocate(chunks[0].offset, DEFAULT_LAYOUT,),
                is_ok
            );
        }

        let chunk = sut_create.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_ok);
    }

    #[test]
    fn allocation_with_creator_and_client_works<Sut: SharedMemory<DefaultAllocator>>() {
        let name = generate_name();
        let mut chunks = vec![];

        let sut_create = Sut::Builder::new(&name)
            .size(DEFAULT_SIZE)
            .create(&SHM_CONFIG)
            .unwrap();

        for _ in 0..NUMBER_OF_CHUNKS / 2 {
            let chunk = sut_create.allocate(DEFAULT_LAYOUT);
            assert_that!(chunk, is_ok);
            chunks.push(chunk.unwrap());
        }

        let sut_open = Sut::Builder::new(&name).open().unwrap();
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
            assert_that!(
                sut_create.deallocate(chunks[0].offset, DEFAULT_LAYOUT,),
                is_ok
            );
        }

        let chunk = sut_open.allocate(DEFAULT_LAYOUT);
        assert_that!(chunk, is_ok);
    }

    #[test]
    fn list_shm_works<Sut: SharedMemory<DefaultAllocator>>() {
        let mut storage_names = vec![];
        let mut storages = vec![];
        const LIMIT: usize = 8;

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len i);
            storage_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&storage_names[i]), eq Ok(false));
            storages.push(
                Sut::Builder::new(&storage_names[i])
                    .size(DEFAULT_SIZE)
                    .create(&SHM_CONFIG)
                    .unwrap(),
            );
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&storage_names[i]), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list().unwrap();
            assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len i + 1);
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

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len LIMIT);

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&storage_names[i])}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&storage_names[i])}, eq Ok(false));
        }

        std::mem::forget(storages);

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_storages_separated<Sut: SharedMemory<DefaultAllocator>>() {
        let config_1 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s1") });
        let config_2 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s2") });

        let storage_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let mut storage_guard_1 = Sut::Builder::new(&storage_name)
            .size(DEFAULT_SIZE)
            .config(&config_1)
            .create(&SHM_CONFIG)
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let mut storage_guard_2 = Sut::Builder::new(&storage_name)
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
        assert_that!(*config.get_suffix(), eq DEFAULT_SUFFIX);
        assert_that!(*config.get_path_hint(), eq DEFAULT_PATH_HINT);
    }

    #[instantiate_tests(<iceoryx2_cal::shared_memory::posix::Memory<DefaultAllocator>>)]
    mod posix {}

    #[instantiate_tests(<iceoryx2_cal::shared_memory::process_local::Memory<DefaultAllocator>>)]
    mod process_local {}
}
