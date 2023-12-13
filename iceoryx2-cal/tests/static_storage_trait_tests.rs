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
mod static_storage {
    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::static_storage::*;
    use std::sync::Mutex;

    /// The list all storage tests requires that all other tests are not interfering and therefore
    /// we cannot let them run concurrently.
    static TEST_MUTEX: Mutex<u8> = Mutex::new(0);

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"static_storage_tests_").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_string()
                .as_bytes(),
        )
        .unwrap();
        file
    }

    #[test]
    fn create_and_read_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "some storage content".to_string();

        let storage_guard = Sut::Builder::new(&storage_name)
            .create(unsafe { content.as_mut_vec() }.as_slice())
            .unwrap();

        assert_that!(*storage_guard.name(), eq storage_name);

        let storage_reader = Sut::Builder::new(&storage_name).open().unwrap();

        assert_that!(*storage_reader.name(), eq storage_name);
        let content_len = content.len() as u64;
        assert_that!(storage_reader, len content_len);

        let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
        storage_reader
            .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
            .unwrap();
        assert_that!(read_content, eq content);
    }

    #[test]
    fn open_non_existing_fails<Sut: StaticStorage>() {
        let storage_name = generate_name();

        let _test_guard = TEST_MUTEX.lock();
        let storage_reader = Sut::Builder::new(&storage_name).open();

        assert_that!(storage_reader, is_err);
        assert_that!(
            storage_reader.err().unwrap(), eq
            StaticStorageOpenError::DoesNotExist
        );
    }

    #[test]
    fn when_storage_guard_goes_out_of_scope_storage_is_removed<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "some storage content".to_string();
        let storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());

        drop(storage_guard);
        let result = Sut::Builder::new(&storage_name).open();
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq StaticStorageOpenError::DoesNotExist);
    }

    #[test]
    fn cannot_create_same_storage_twice<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "some storage content".to_string();
        let _storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());
        let result =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());

        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            StaticStorageCreateError::AlreadyExists
        );
    }

    #[test]
    fn after_reader_is_created_guard_can_be_dropped<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "another\nfunky\nstorage".to_string();
        let storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());

        let storage_reader = Sut::Builder::new(&storage_name).open().unwrap();
        drop(storage_guard);

        let content_len = content.len() as u64;
        assert_that!(storage_reader, len content_len);

        let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
        storage_reader
            .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
            .unwrap();
        assert_that!(read_content, eq content.clone());

        let storage_reader = Sut::Builder::new(&storage_name).open();
        assert_that!(storage_reader, is_err);
        assert_that!(
            storage_reader.err().unwrap(), eq
            StaticStorageOpenError::DoesNotExist
        );

        let storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());
        assert_that!(storage_guard, is_ok);
    }

    #[test]
    fn last_open_reader_drops_storage<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "another\nfunky\nstorage".to_string();
        let storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());

        let storage_reader = Sut::Builder::new(&storage_name).open().unwrap();
        drop(storage_guard);
        drop(storage_reader);

        let storage_reader = Sut::Builder::new(&storage_name).open();
        assert_that!(storage_reader, is_err);
        assert_that!(
            storage_reader.err().unwrap(), eq
            StaticStorageOpenError::DoesNotExist
        );
    }

    #[test]
    fn read_same_storage_multiple_times_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "another\nfunky\nstorage".to_string();
        let storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());

        let storage_reader_alt = Sut::Builder::new(&storage_name).open().unwrap();
        let storage_reader = Sut::Builder::new(&storage_name).open().unwrap();
        drop(storage_guard);

        let content_len = content.len() as u64;
        assert_that!(storage_reader, len content_len);
        assert_that!(storage_reader_alt, len content_len);

        let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
        storage_reader
            .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
            .unwrap();
        assert_that!(read_content, eq content);

        storage_reader_alt
            .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
            .unwrap();
        assert_that!(read_content, eq content);
    }

    #[test]
    fn read_with_insufficient_buffer_fails<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let mut content = "some \nfuu\n cont\tent".to_string();
        let _storage_guard =
            Sut::Builder::new(&storage_name).create(unsafe { content.as_mut_vec() }.as_slice());

        let storage_reader = Sut::Builder::new(&storage_name).open().unwrap();

        let content_len = content.len() as u64;
        assert_that!(storage_reader, len content_len);

        let mut read_content = String::new();
        let result = storage_reader.read(unsafe { read_content.as_mut_vec() }.as_mut_slice());
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            StaticStorageReadError::BufferTooSmall
        );
    }

    #[test]
    fn list_all_storages_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        const NUMBER_OF_STORAGES: u64 = 12;

        let mut storages = vec![];
        for i in 0..NUMBER_OF_STORAGES {
            assert_that!(Sut::list().unwrap(), len i as usize);
            let storage_name = generate_name();

            let mut content = "some \nfuu\n cont\tent".to_string();
            storages.push(
                Sut::Builder::new(&storage_name)
                    .create(unsafe { content.as_mut_vec() }.as_slice())
                    .unwrap(),
            );

            let contents = Sut::list().unwrap();
            assert_that!(Sut::list().unwrap(), len(i + 1) as usize);

            let contains = |s| {
                for entry in &storages {
                    if *entry.name() == s {
                        return true;
                    }
                }
                false
            };

            for entry in contents {
                assert_that!(contains(entry), eq true);
            }
        }

        assert_that!(Sut::list().unwrap(), len NUMBER_OF_STORAGES as usize);

        for i in 0..NUMBER_OF_STORAGES {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(storages[i as usize].name())}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(storages[i as usize].name())}, eq Ok(false));
        }

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
    }

    #[test]
    fn does_exist_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        const NUMBER_OF_STORAGES: u64 = 12;
        const NUMBER_OF_LOCKED_STORAGES: u64 = 12;

        let mut storages = vec![];
        let mut locked_storages = vec![];
        let mut storage_names = vec![];
        let mut locked_storage_names = vec![];

        for _i in 0..NUMBER_OF_STORAGES {
            let storage_name = generate_name();
            storage_names.push(storage_name);

            let mut content = "some \nfuu\n cont\tent".to_string();
            storages.push(
                Sut::Builder::new(&storage_name)
                    .create(unsafe { content.as_mut_vec() }.as_slice())
                    .unwrap(),
            );
        }

        for _i in 0..NUMBER_OF_LOCKED_STORAGES {
            let storage_name = generate_name();
            locked_storage_names.push(storage_name);
            locked_storages.push(Sut::Builder::new(&storage_name).create_locked().unwrap());
        }

        for entry in &storage_names {
            assert_that!(Sut::does_exist(entry), eq Ok(true));
        }

        for entry in &locked_storage_names {
            assert_that!(Sut::does_exist(entry), eq Err(NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp));
        }

        drop(storages);
        drop(locked_storages);

        for entry in &storage_names {
            assert_that!(Sut::does_exist(entry), eq Ok(false));
        }

        for entry in &locked_storage_names {
            assert_that!(Sut::does_exist(entry), eq Ok(false));
        }
    }

    #[test]
    fn create_locked_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let content = "whooo hoo hoo".to_string();

        let storage_guard = Sut::Builder::new(&storage_name).create_locked();

        assert_that!(storage_guard, is_ok);
        assert_that!(Sut::does_exist(&storage_name), eq Err(NamedConceptDoesExistError::UnderlyingResourcesBeingSetUp));
        assert_that!(*storage_guard.as_ref().unwrap().name(), eq storage_name);

        let storage_reader = Sut::Builder::new(&storage_name).open();
        assert_that!(storage_reader, is_err);
        assert_that!(
            storage_reader.err().unwrap(), eq
            StaticStorageOpenError::IsLocked
        );

        let storage_guard = storage_guard.unwrap().unlock(content.as_bytes());
        assert_that!(storage_guard, is_ok);
        assert_that!(Sut::does_exist(&storage_name), eq Ok(true));

        let storage_reader = Sut::Builder::new(&storage_name).open().unwrap();

        assert_that!(*storage_reader.name(), eq storage_name);
        let content_len = content.len() as u64;
        assert_that!(storage_reader, len content_len);

        let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
        storage_reader
            .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
            .unwrap();
        assert_that!(read_content, eq content);
    }

    #[test]
    fn releasing_ownership_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let content = "whooo hoo hoo".to_string();

        let mut storage_guard = Sut::Builder::new(&storage_name)
            .create(content.as_bytes())
            .unwrap();

        storage_guard.release_ownership();
        drop(storage_guard);

        assert_that!(Sut::does_exist(&storage_name), eq Ok(true));
        unsafe { Sut::remove(&storage_name).unwrap() };
        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));
    }

    #[test]
    fn create_without_ownership_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let content = "whooo hoo hoo".to_string();

        let storage_guard = Sut::Builder::new(&storage_name)
            .has_ownership(false)
            .create(content.as_bytes())
            .unwrap();

        drop(storage_guard);

        assert_that!(Sut::does_exist(&storage_name), eq Ok(true));
        assert_that!(unsafe { Sut::remove(&storage_name) }, eq Ok(true));
        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));
    }

    #[test]
    fn acquire_ownership_works<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();
        let storage_name = generate_name();

        let content = "whooo hoo hoo".to_string();

        let mut storage_guard = Sut::Builder::new(&storage_name)
            .has_ownership(false)
            .create(content.as_bytes())
            .unwrap();

        storage_guard.acquire_ownership();
        drop(storage_guard);

        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));
    }

    #[test]
    fn custom_suffix_keeps_storages_separated<Sut: StaticStorage>() {
        let _test_guard = TEST_MUTEX.lock();

        let config_1 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".static_storage_1") });
        let config_2 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".static_storage_2") });

        let storage_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let mut storage_guard_1 = Sut::Builder::new(&storage_name)
            .config(&config_1)
            .create(b"")
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let mut storage_guard_2 = Sut::Builder::new(&storage_name)
            .config(&config_2)
            .create(b"")
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

        assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: StaticStorage>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq DEFAULT_SUFFIX);
        assert_that!(*config.get_path_hint(), eq DEFAULT_PATH_HINT);
    }

    #[instantiate_tests(<iceoryx2_cal::static_storage::file::Storage>)]
    mod file {}

    #[instantiate_tests(<iceoryx2_cal::static_storage::process_local::Storage>)]
    mod process_local {}
}
