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
    use core::sync::atomic::{AtomicI64, Ordering};
    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_elementary_traits::allocator::*;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_cal::dynamic_storage::*;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;
    use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY;
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    const TIMEOUT: Duration = Duration::from_millis(100);

    #[derive(Debug)]
    struct TestData {
        value: AtomicI64,
        supplementary_ptr: *mut u8,
        supplementary_len: usize,
        _lifetime_tracker: Option<LifetimeTracker>,
    }

    impl TestData {
        fn new(value: i64) -> Self {
            Self {
                value: AtomicI64::new(value),
                supplementary_ptr: core::ptr::null_mut::<u8>(),
                supplementary_len: 0,
                _lifetime_tracker: None,
            }
        }

        fn new_with_lifetime_tracking(value: i64) -> Self {
            Self {
                value: AtomicI64::new(value),
                supplementary_ptr: core::ptr::null_mut::<u8>(),
                supplementary_len: 0,
                _lifetime_tracker: Some(LifetimeTracker::new()),
            }
        }
    }

    unsafe impl Send for TestData {}
    unsafe impl Sync for TestData {}

    #[test]
    fn create_and_read_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(123))
            .unwrap();

        assert_that!(*sut.name(), eq storage_name);

        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

        assert_that!(*sut2.name(), eq storage_name);

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 123);
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 123);

        sut2.get().value.store(456, Ordering::Relaxed);

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 456);
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 456);
    }

    #[test]
    fn open_non_existing_fails<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name).config(&config).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn when_storage_goes_out_of_scope_storage_is_removed<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(123));
        drop(sut);

        let sut = Sut::Builder::new(&storage_name).config(&config).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn cannot_create_same_storage_twice<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut1 = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(123));
        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(123));

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            DynamicStorageCreateError::AlreadyExists
        );
    }

    #[test]
    fn after_storage_is_opened_creator_can_be_dropped<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        test_requires!(Sut::does_support_persistency());

        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(123))
            .unwrap();

        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

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
    fn create_and_multiple_openers_works<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        const NUMBER_OF_OPENERS: u64 = 64;
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(789))
            .unwrap();

        let mut sut_vec = vec![];
        for _i in 0..NUMBER_OF_OPENERS {
            sut_vec.push(
                Sut::Builder::new(&storage_name)
                    .config(&config)
                    .open()
                    .unwrap(),
            );
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
    fn release_ownership_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        test_requires!(Sut::does_support_persistency());
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(9887))
            .unwrap();

        assert_that!(sut.has_ownership(), eq true);
        sut.release_ownership();
        assert_that!(sut.has_ownership(), eq false);
        drop(sut);

        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 9887);

        assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(true));
        drop(sut2);

        let sut2 = Sut::Builder::new(&storage_name).open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn create_non_owning_storage_works<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        test_requires!(Sut::does_support_persistency());
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .has_ownership(false)
            .config(&config)
            .create(TestData::new(9887))
            .unwrap();

        assert_that!(sut.has_ownership(), eq false);

        drop(sut);

        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 9887);

        assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(true));
        drop(sut2);

        let sut2 = Sut::Builder::new(&storage_name).config(&config).open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn acquire_ownership_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        test_requires!(Sut::does_support_persistency());
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .has_ownership(false)
            .config(&config)
            .create(TestData::new(9887))
            .unwrap();

        sut.acquire_ownership();

        assert_that!(sut.has_ownership(), eq true);
        drop(sut);
        assert_that!(Sut::does_exist_cfg(&storage_name, &config).unwrap(), eq false);
    }

    #[test]
    fn does_exist_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(false));

        let additional_size: usize = 256;
        let sut = Sut::Builder::new(&storage_name)
            .supplementary_size(additional_size)
            .config(&config)
            .create(TestData::new(9887))
            .unwrap();
        let _sut2 = Sut::Builder::new(&storage_name)
            .supplementary_size(additional_size)
            .config(&config)
            .open()
            .unwrap();

        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));
        drop(sut);
        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(false));
    }

    #[test]
    fn has_ownership_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        test_requires!(Sut::does_support_persistency());

        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(123))
            .unwrap();

        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();

        assert_that!(sut.has_ownership(), eq true);
        assert_that!(sut2.has_ownership(), eq false);

        sut.release_ownership();
        assert_that!(sut.has_ownership(), eq false);
        drop(sut);
        drop(sut2);
        assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(true));
    }

    #[test]
    fn create_and_initialize_works<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .supplementary_size(134)
            .initializer(|value, allocator| {
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
            .create(TestData::new(123))
            .unwrap();

        assert_that!(sut.get().value.load(Ordering::Relaxed), eq 8912);
        assert_that!(sut.get().supplementary_len, eq 134);
        for i in 0..134 {
            assert_that!(
                unsafe { *sut.get().supplementary_ptr.offset(i as isize) },
                eq 134 - i
            );
        }

        let sut2 = Sut::Builder::new(&storage_name)
            .config(&config)
            .open()
            .unwrap();
        assert_that!(sut2.get().value.load(Ordering::Relaxed), eq 8912);
        assert_that!(sut2.get().supplementary_len, eq 134);
        for i in 0..134 {
            assert_that!(
                unsafe { *sut2.get().supplementary_ptr.offset(i as isize) },
                eq 134 - i
            );
        }
    }

    #[ignore] // TODO: iox2-671 enable this test when the concurrency issue is fixed.
    #[test]
    fn initialization_blocks_other_openers<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let _watchdog = Watchdog::new();
        let barrier_1 = Arc::new(Barrier::new(2));
        let barrier_2 = barrier_1.clone();

        std::thread::scope(|s| {
            let tstorage_name = storage_name.clone();
            let config_1 = config.clone();
            s.spawn(move || {
                barrier_1.wait();
                let _sut = Sut::Builder::new(&tstorage_name)
                    .config(&config_1)
                    .supplementary_size(0)
                    .has_ownership(false)
                    .initializer(|value, _| {
                        std::thread::sleep(TIMEOUT);
                        value.value.store(789, Ordering::Relaxed);
                        true
                    })
                    .create(TestData::new(123))
                    .unwrap();
            });

            let tstorage_name = storage_name.clone();
            let config_2 = config.clone();
            s.spawn(move || {
                barrier_2.wait();
                loop {
                let sut2 = Sut::Builder::new(&tstorage_name).config(&config_2).open();
                if sut2.is_err() {
                    let err = sut2.err().unwrap();
                    assert_that!(err == DynamicStorageOpenError::DoesNotExist || err == DynamicStorageOpenError::InitializationNotYetFinalized, eq true);
                } else {
                    assert_that!(sut2.unwrap().get().value.load(Ordering::Relaxed), eq 789);
                    break;
                }
            }});
        });

        if POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY {
            assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(true));
            assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(true));
        }
    }

    #[test]
    fn initialization_timeout_blocks_for_at_least_timeout<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let barrier = Arc::new(Barrier::new(2));
        let _watchdog = Watchdog::new();

        std::thread::scope(|s| {
            let tstorage_name = storage_name.clone();
            let config_1 = config.clone();
            let barrier_1 = barrier.clone();
            s.spawn(move || {
                let _sut = Sut::Builder::new(&tstorage_name)
                    .config(&config_1)
                    .supplementary_size(0)
                    .initializer(|_, _| {
                        barrier_1.wait();
                        std::thread::sleep(TIMEOUT * 2);
                        true
                    })
                    .create(TestData::new(123))
                    .unwrap();
            });

            let config_2 = config.clone();
            let barrier_2 = barrier.clone();
            s.spawn(move || {
                barrier_2.wait();
                let start = Instant::now();
                let _sut = Sut::Builder::new(&storage_name)
                    .config(&config_2)
                    .timeout(TIMEOUT)
                    .open();

                assert_that!(start.elapsed(), time_at_least TIMEOUT);
            });
        });
    }

    #[test]
    fn create_fails_when_initialization_fails<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .supplementary_size(134)
            .initializer(|_, _| false)
            .config(&config)
            .create(TestData::new(123));

        assert_that!(sut, is_err);
        assert_that!(
            sut.err().unwrap(), eq
            DynamicStorageCreateError::InitializationFailed
        );

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&storage_name, &config), eq Ok(false));
        assert_that!(unsafe { <Sut as NamedConceptMgmt>::remove_cfg(&storage_name, &config) }, eq Ok(false));
    }

    #[test]
    fn list_storages_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        let mut sut_names = vec![];
        let mut suts = vec![];
        const LIMIT: usize = 5;
        let config = generate_isolated_config::<Sut>();

        for i in 0..LIMIT {
            assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len i );
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(false));
            suts.push(
                Sut::Builder::new(&sut_names[i])
                    .supplementary_size(134)
                    .config(&config)
                    .create(TestData::new(123)),
            );
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(true));

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

            for name in &sut_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len LIMIT);

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(false));
        }

        core::mem::forget(suts);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_storages_separated<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let config = generate_isolated_config::<Sut>();
        let config_1 = config
            .clone()
            .suffix(unsafe { &FileName::new_unchecked(b".s1") });
        let config_2 = config.suffix(unsafe { &FileName::new_unchecked(b".s2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::Builder::new(&sut_name)
            .config(&config_1)
            .supplementary_size(134)
            .create(TestData::new(123))
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config_2)
            .supplementary_size(134)
            .create(TestData::new(123))
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq sut_name);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq sut_name);

        assert_that!(*sut_1.name(), eq sut_name);
        assert_that!(*sut_2.name(), eq sut_name);

        core::mem::forget(sut_1);
        core::mem::forget(sut_2);

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
        assert_that!(*config.get_path_hint(), eq Sut::default_path_hint());
        assert_that!(*config.get_prefix(), eq Sut::default_prefix());
    }

    #[test]
    fn open_or_create_works<Sut: DynamicStorage<TestData>, WrongTypeSut: DynamicStorage<u64>>() {
        let sut_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(Sut::does_exist_cfg(&sut_name, &config), eq Ok(false));
        let sut_1 = Sut::Builder::new(&sut_name)
            .config(&config)
            .open_or_create(TestData::new(123));
        assert_that!(sut_1, is_ok);
        assert_that!(Sut::does_exist_cfg(&sut_name, &config), eq Ok(true));

        let sut_2 = Sut::Builder::new(&sut_name)
            .config(&config)
            .open_or_create(TestData::new(123));
        assert_that!(sut_2, is_ok);

        drop(sut_2);
        drop(sut_1);
        assert_that!(Sut::does_exist_cfg(&sut_name, &config), eq Ok(false));
    }

    #[test]
    fn by_default_when_storage_is_removed_it_calls_drop<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let state = LifetimeTracker::start_tracking();

        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new_with_lifetime_tracking(123))
            .unwrap();

        assert_that!(sut.has_ownership(), eq true);
        drop(sut);

        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn when_drop_on_destruction_is_disabled_remove_does_not_call_drop<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let state = LifetimeTracker::start_tracking();

        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .call_drop_on_destruction(false)
            .create(TestData::new_with_lifetime_tracking(123))
            .unwrap();

        assert_that!(sut.has_ownership(), eq true);
        drop(sut);

        assert_that!(state.number_of_living_instances(), eq 1);
    }

    #[test]
    fn when_storage_is_persistent_it_does_not_call_drop<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let state = LifetimeTracker::start_tracking();

        test_requires!(Sut::does_support_persistency());

        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new_with_lifetime_tracking(123))
            .unwrap();
        sut.release_ownership();
        drop(sut);

        assert_that!(state.number_of_living_instances(), eq 1);
        assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(true));
    }

    #[test]
    fn explicit_remove_of_persistent_storage_calls_drop<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let state = LifetimeTracker::start_tracking();

        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new_with_lifetime_tracking(123))
            .unwrap();
        sut.release_ownership();
        // it leaks a memory mapping here but this we want explicitly to test remove also
        // for platforms that do not support persistent dynamic storage
        core::mem::forget(sut);

        assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(true));
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn remove_storage_with_unfinished_initialization_does_call_drop<
        Sut: DynamicStorage<TestData> + 'static,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let state = LifetimeTracker::start_tracking();
        let config = generate_isolated_config::<Sut>();

        if core::any::TypeId::of::<Sut>()
            // skip process local test since the process locality ensures that an initializer
            // never dies
            != core::any::TypeId::of::<iceoryx2_cal::dynamic_storage::process_local::Storage<TestData>>(
            )
        {
            let storage_name = generate_name();

            let _ = Sut::Builder::new(&storage_name)
                .has_ownership(false)
                .config(&config)
                .initializer(|_, _| false)
                .create(TestData::new_with_lifetime_tracking(0));

            assert_that!(unsafe { Sut::remove_cfg(&storage_name, &config) }, eq Ok(false));
            assert_that!(state.number_of_living_instances(), eq 0);
        }
    }

    #[test]
    fn dynamic_storage_with_wrong_type_does_not_exist<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let wrong_type_config = WrongTypeSut::Configuration::default()
            .prefix(config.get_prefix())
            .suffix(config.get_suffix())
            .path_hint(config.get_path_hint());
        let sut = WrongTypeSut::Builder::new(&storage_name)
            .config(&wrong_type_config)
            .create(123);
        assert_that!(sut, is_ok);
        assert_that!(Sut::does_exist_cfg(&storage_name, &config), eq Ok(false));
    }

    #[test]
    fn different_types_are_separated_also_in_list<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        const NUMBER_OF_TESTDATA_STORAGES: usize = 10;
        const NUMBER_OF_U64_STORAGES: usize = 15;

        let mut testdata_storages = vec![];
        let mut testdata_storages_names = vec![];
        let mut u64_storages = vec![];
        let mut u64_storages_names = vec![];

        let config = generate_isolated_config::<Sut>();
        let wrong_type_config = WrongTypeSut::Configuration::default()
            .prefix(config.get_prefix())
            .suffix(config.get_suffix())
            .path_hint(config.get_path_hint());
        for _ in 0..NUMBER_OF_TESTDATA_STORAGES {
            let storage_name = generate_name();
            testdata_storages.push(
                Sut::Builder::new(&storage_name)
                    .config(&config)
                    .create(TestData::new(123))
                    .unwrap(),
            );
            testdata_storages_names.push(storage_name.clone());

            u64_storages.push(
                WrongTypeSut::Builder::new(&storage_name)
                    .config(&wrong_type_config)
                    .create(34)
                    .unwrap(),
            );

            u64_storages_names.push(storage_name);
        }

        for _ in 0..NUMBER_OF_U64_STORAGES {
            let storage_name = generate_name();
            u64_storages.push(
                WrongTypeSut::Builder::new(&storage_name)
                    .config(&wrong_type_config)
                    .create(21)
                    .unwrap(),
            );
            u64_storages_names.push(storage_name);
        }

        let testdata_list = Sut::list_cfg(&config).unwrap();
        let u64_list = WrongTypeSut::list_cfg(&wrong_type_config).unwrap();

        assert_that!(testdata_list, len testdata_storages.len());
        assert_that!(u64_list, len u64_storages.len());

        for name in testdata_list {
            assert_that!(testdata_storages_names, contains name);
        }

        for name in u64_list {
            assert_that!(u64_storages_names, contains name);
        }
    }

    #[test]
    fn opening_dynamic_storage_with_wrong_type_fails<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let wrong_type_config = WrongTypeSut::Configuration::default()
            .prefix(config.get_prefix())
            .suffix(config.get_suffix())
            .path_hint(config.get_path_hint());

        let _sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(1234))
            .unwrap();
        let sut = WrongTypeSut::Builder::new(&storage_name)
            .config(&wrong_type_config)
            .open();
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::DoesNotExist);
    }

    #[test]
    fn removing_dynamic_storage_with_wrong_type_fails<
        Sut: DynamicStorage<TestData>,
        WrongTypeSut: DynamicStorage<u64>,
    >() {
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let wrong_type_config = WrongTypeSut::Configuration::default()
            .prefix(config.get_prefix())
            .suffix(config.get_suffix())
            .path_hint(config.get_path_hint());
        let _sut = Sut::Builder::new(&storage_name)
            .config(&config)
            .create(TestData::new(1234))
            .unwrap();
        assert_that!(unsafe { WrongTypeSut::remove_cfg(&storage_name, &wrong_type_config) }, eq Ok(false));
    }

    #[instantiate_tests(<iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<TestData>,
                         iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<u64>>)]
    mod posix_shared_memory {}

    #[instantiate_tests(<iceoryx2_cal::dynamic_storage::process_local::Storage<TestData>,
                         iceoryx2_cal::dynamic_storage::process_local::Storage<u64>>)]
    mod process_local {}
}
