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
mod dynamic_storage {
    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_elementary::allocator::*;
    use iceoryx2_bb_elementary::math::ToB64;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_cal::dynamic_storage::*;
    use iceoryx2_cal::named_concept::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"test_").unwrap();
        file.push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
            .unwrap();
        file
    }

    #[derive(Debug)]
    struct TestData {
        value: AtomicI64,
        supplementary_ptr: *mut u8,
        supplementary_len: usize,
    }

    impl TestData {
        fn new(value: i64) -> Self {
            Self {
                value: AtomicI64::new(value),
                supplementary_ptr: std::ptr::null_mut::<u8>(),
                supplementary_len: 0,
            }
        }
    }

    unsafe impl Send for TestData {}
    unsafe impl Sync for TestData {}

    #[test]
    fn create_and_read_works<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name)
            .create(TestData::new(123))
            .unwrap();

        assert_that!(*sut.name(), eq storage_name);

        let sut2 = Sut::Builder::new(&storage_name).open().unwrap();

        assert_that!(*sut2.name(), eq storage_name);

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 123);
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 123);

        sut2.get().value.store(456, Ordering::Relaxed);

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 456);
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 456);
    }

    #[test]
    fn open_non_existing_fails<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();
        let sut = Sut::Builder::new(&storage_name).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn when_storage_goes_out_of_scope_storage_is_removed<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name).create(TestData::new(123));
        drop(sut);

        let sut = Sut::Builder::new(&storage_name).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn cannot_create_same_storage_twice<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        let _sut1 = Sut::Builder::new(&storage_name).create(TestData::new(123));
        let sut2 = Sut::Builder::new(&storage_name).create(TestData::new(123));

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            DynamicStorageCreateError::AlreadyExists
        );
    }

    #[test]
    fn after_storage_is_opened_creator_can_be_dropped<Sut: DynamicStorage<TestData>>() {
        test_requires!(Sut::does_support_persistency());

        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name)
            .create(TestData::new(123))
            .unwrap();

        let sut2 = Sut::Builder::new(&storage_name).open().unwrap();

        drop(sut);

        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 123);

        sut2.get().value.store(456, Ordering::Relaxed);

        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 456);

        let sut3 = Sut::Builder::new(&storage_name).open();

        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
        drop(sut2);

        let sut = Sut::Builder::new(&storage_name).create(TestData::new(123));
        assert_that!(sut, is_ok);
    }

    #[test]
    fn create_and_multiple_openers_works<Sut: DynamicStorage<TestData>>() {
        const NUMBER_OF_OPENERS: u64 = 64;
        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name)
            .create(TestData::new(789))
            .unwrap();

        let mut sut_vec = vec![];
        for _i in 0..NUMBER_OF_OPENERS {
            sut_vec.push(Sut::Builder::new(&storage_name).open().unwrap());
        }

        for i in 0..NUMBER_OF_OPENERS {
            assert_that!(sut_vec[i as usize].get().value.load(Ordering::Relaxed), eq 789);
        }

        sut_vec[0].get().value.store(5001, Ordering::Relaxed);

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 5001);
        for i in 0..NUMBER_OF_OPENERS {
            assert_that!(
                sut_vec[i as usize].get().value.load(Ordering::Relaxed), eq
                5001
            );
        }
    }

    #[test]
    fn release_ownership_works<Sut: DynamicStorage<TestData>>() {
        test_requires!(Sut::does_support_persistency());
        let storage_name = generate_name();

        let mut sut = Sut::Builder::new(&storage_name)
            .create(TestData::new(9887))
            .unwrap();

        assert_that!(sut.has_ownership(), eq true);
        sut.release_ownership();
        assert_that!(sut.has_ownership(), eq false);
        drop(sut);

        let sut2 = Sut::Builder::new(&storage_name).open().unwrap();
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 9887);

        assert_that!(unsafe { Sut::remove(&storage_name) }, eq Ok(true));
        drop(sut2);

        let sut2 = Sut::Builder::new(&storage_name).open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn create_non_owning_storage_works<Sut: DynamicStorage<TestData>>() {
        test_requires!(Sut::does_support_persistency());
        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name)
            .has_ownership(false)
            .create(TestData::new(9887))
            .unwrap();

        assert_that!(sut.has_ownership(), eq false);

        drop(sut);

        let sut2 = Sut::Builder::new(&storage_name).open().unwrap();
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 9887);

        assert_that!(unsafe { Sut::remove(&storage_name) }, eq Ok(true));
        drop(sut2);

        let sut2 = Sut::Builder::new(&storage_name).open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn acquire_ownership_works<Sut: DynamicStorage<TestData>>() {
        test_requires!(Sut::does_support_persistency());
        let storage_name = generate_name();

        let mut sut = Sut::Builder::new(&storage_name)
            .has_ownership(false)
            .create(TestData::new(9887))
            .unwrap();

        sut.acquire_ownership();

        assert_that!(sut.has_ownership(), eq true);
        drop(sut);
        assert_that!(Sut::does_exist(&storage_name).unwrap(), eq false);
    }

    #[test]
    fn does_exist_works<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));

        let additional_size: usize = 256;
        let sut = Sut::Builder::new(&storage_name)
            .supplementary_size(additional_size)
            .create(TestData::new(9887))
            .unwrap();
        let _sut2 = Sut::Builder::new(&storage_name)
            .supplementary_size(additional_size)
            .open()
            .unwrap();

        assert_that!(Sut::does_exist(&storage_name), eq Ok(true));
        drop(sut);
        assert_that!(Sut::does_exist(&storage_name), eq Ok(false));
    }

    #[test]
    fn has_ownership_works<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        let mut sut = Sut::Builder::new(&storage_name)
            .create(TestData::new(123))
            .unwrap();

        let sut2 = Sut::Builder::new(&storage_name).open().unwrap();

        assert_that!(sut.has_ownership(), eq true);
        assert_that!(sut2.has_ownership(), eq false);

        sut.release_ownership();
        assert_that!(sut.has_ownership(), eq false);
        drop(sut);
        drop(sut2);
        assert_that!(unsafe { Sut::remove(&storage_name) }, eq Ok(true));
    }

    #[test]
    fn create_and_initialize_works<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name)
            .supplementary_size(134)
            .create_and_initialize(TestData::new(123), |value, allocator| {
                let layout = Layout::from_size_align(134, 1).unwrap();
                let mem = allocator.allocate(layout).unwrap();

                value.value.store(8912, Ordering::Relaxed);
                value.supplementary_ptr = mem.as_ptr() as *mut u8;
                value.supplementary_len = mem.len();

                for i in 0..134 {
                    unsafe {
                        value
                            .supplementary_ptr
                            .offset(i as isize)
                            .write((134 - i) as u8)
                    };
                }
                true
            })
            .unwrap();

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 8912);
        assert_that!(sut.get().supplementary_len, eq 134);
        for i in 0..134 {
            assert_that!(
                unsafe { *sut.get().supplementary_ptr.offset(i as isize) },
                eq 134 - i
            );
        }

        let sut2 = Sut::Builder::new(&storage_name).open().unwrap();
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 8912);
        assert_that!(sut2.get().supplementary_len, eq 134);
        for i in 0..134 {
            assert_that!(
                unsafe { *sut2.get().supplementary_ptr.offset(i as isize) },
                eq 134 - i
            );
        }
    }

    #[test]
    fn create_fails_when_initialization_fails<Sut: DynamicStorage<TestData>>() {
        let storage_name = generate_name();

        let sut = Sut::Builder::new(&storage_name)
            .supplementary_size(134)
            .create_and_initialize(TestData::new(123), |_, _| false);

        assert_that!(sut, is_err);
        assert_that!(
            sut.err().unwrap(), eq
            DynamicStorageCreateError::InitializationFailed
        );

        assert_that!(<Sut as NamedConceptMgmt>::does_exist(&storage_name), eq Ok(false));
        assert_that!(unsafe { <Sut as NamedConceptMgmt>::remove(&storage_name) }, eq Ok(false));
    }

    #[test]
    fn list_storages_works<Sut: DynamicStorage<TestData>>() {
        let mut sut_names = vec![];
        let mut suts = vec![];
        const LIMIT: usize = 5;

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len i );
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(false));
            suts.push(
                Sut::Builder::new(&sut_names[i])
                    .supplementary_size(134)
                    .create_and_initialize(TestData::new(123), |_, _| true),
            );
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(true));

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

            for name in &sut_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len LIMIT);

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&sut_names[i])}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&sut_names[i])}, eq Ok(false));
        }

        std::mem::forget(suts);

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_storages_separated<Sut: DynamicStorage<TestData>>() {
        let config_1 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s1") });
        let config_2 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".s2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::Builder::new(&sut_name)
            .config(&config_1)
            .supplementary_size(134)
            .create_and_initialize(TestData::new(123), |_, _| true)
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config_2)
            .supplementary_size(134)
            .create_and_initialize(TestData::new(123), |_, _| true)
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq sut_name);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq sut_name);

        assert_that!(*sut_1.name(), eq sut_name);
        assert_that!(*sut_2.name(), eq sut_name);

        std::mem::forget(sut_1);
        std::mem::forget(sut_2);

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: DynamicStorage<TestData>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq DEFAULT_SUFFIX);
        assert_that!(*config.get_path_hint(), eq DEFAULT_PATH_HINT);
    }

    #[instantiate_tests(<iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<TestData>>)]
    mod posix_shared_memory {}

    #[instantiate_tests(<iceoryx2_cal::dynamic_storage::process_local::Storage<TestData>>)]
    mod process_local {}
}
