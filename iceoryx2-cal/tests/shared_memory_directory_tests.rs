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

mod shared_memory_directory {
    use core::alloc::Layout;
    use std::sync::Barrier;

    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_cal::shared_memory::SharedMemoryCreateError;
    use iceoryx2_cal::shared_memory_directory::SharedMemoryDirectory;
    use iceoryx2_cal::shm_allocator::ShmAllocator;
    use iceoryx2_cal::testing::generate_name;
    use iceoryx2_cal::{
        shared_memory, shared_memory_directory::SharedMemoryDirectoryCreateFileError,
        shared_memory_directory::SharedMemoryDirectoryCreator, shm_allocator,
    };

    type MgmtShm = shared_memory::posix::Memory<shm_allocator::bump_allocator::BumpAllocator>;
    type Allocator = shm_allocator::pool_allocator::PoolAllocator;
    type DataShm = shared_memory::posix::Memory<Allocator>;

    #[test]
    fn create_works() {
        let name = generate_name();
        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            );

        assert_that!(sut, is_ok);
        assert_that!(sut.unwrap().size(), ge 1024 * 1024 );
    }

    #[test]
    fn create_same_dir_twice_fails() {
        let name = generate_name();
        let _sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            );

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            );

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq SharedMemoryCreateError::AlreadyExists);
    }

    #[test]
    fn when_out_of_scope_dir_is_removed() {
        let name = generate_name();
        {
            let _sut = SharedMemoryDirectoryCreator::new(&name)
                .size(1024 * 1024)
                .create::<MgmtShm, Allocator, DataShm>(
                    &<Allocator as ShmAllocator>::Configuration::default(),
                );
        }

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            );

        assert_that!(sut, is_ok);
    }

    #[test]
    fn persistent_directory_is_not_removed() {
        test_requires!(
            SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::does_support_persistency()
        );

        let name = generate_name();
        {
            let _sut = SharedMemoryDirectoryCreator::new(&name)
                .size(1024 * 1024)
                .is_persistent(true)
                .create::<MgmtShm, Allocator, DataShm>(
                    &<Allocator as ShmAllocator>::Configuration::default(),
                );
        }

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            );

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq SharedMemoryCreateError::AlreadyExists);
        assert_that!(unsafe { SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::remove(&name) }, eq Ok(true));
    }

    #[test]
    fn does_exist_works() {
        let name = generate_name();

        assert_that!(SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::does_exist(&name), eq Ok(false));

        {
            let _sut = SharedMemoryDirectoryCreator::new(&name)
                .size(1024 * 1024)
                .create::<MgmtShm, Allocator, DataShm>(
                    &<Allocator as ShmAllocator>::Configuration::default(),
                );
            assert_that!(SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::does_exist(&name), eq Ok(true));
        }

        assert_that!(SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::does_exist(&name) , eq Ok(false));
    }

    #[test]
    fn remove_works() {
        let name = generate_name();

        assert_that!(unsafe { SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::remove(&name) }, eq Ok(false));

        let _sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .is_persistent(true)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            );

        assert_that!(unsafe { SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::remove(&name) }, eq Ok(true));
        assert_that!(unsafe { SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::remove(&name) }, eq Ok(false));
    }

    #[test]
    fn list_works() {
        const NUMBER_OF_INSTANCES: usize = 8;

        let mut names = vec![];
        for _ in 0..NUMBER_OF_INSTANCES {
            let name = generate_name();
            names.push(name.clone());

            let _sut = SharedMemoryDirectoryCreator::new(&name)
                .size(1024 * 1024)
                .is_persistent(true)
                .create::<MgmtShm, Allocator, DataShm>(
                    &<Allocator as ShmAllocator>::Configuration::default(),
                );
        }

        let list_of_dirs = SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::list().unwrap();

        for name in &names {
            assert_that!(list_of_dirs, contains * name);
            unsafe { SharedMemoryDirectory::<MgmtShm, Allocator, DataShm>::remove(name).unwrap() };
        }
    }

    #[test]
    fn persistent_file_is_persistent() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let file_name = generate_name();
        assert_that!(sut.does_file_exist(&file_name), eq false);
        let file = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .is_persistent(true)
            .create(&file_name, |_| {})
            .unwrap();
        assert_that!(sut.does_file_exist(&file_name), eq true);
        assert_that!(file.is_persistent(), eq true);
        drop(file);
        assert_that!(sut.does_file_exist(&file_name), eq true);
        assert_that!(sut.remove_file(&file_name), eq true);
    }

    #[test]
    fn non_persistent_file_will_be_removed_by_guard() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let file_name = generate_name();
        assert_that!(sut.does_file_exist(&file_name), eq false);
        let file = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |_| {})
            .unwrap();
        assert_that!(sut.does_file_exist(&file_name), eq true);
        assert_that!(file.is_persistent(), eq false);
        drop(file);
        assert_that!(sut.does_file_exist(&file_name), eq false);
        assert_that!(sut.remove_file(&file_name), eq false);
    }

    #[test]
    fn cannot_create_same_file_twice() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let file_name = generate_name();
        let _file = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |_| {})
            .unwrap();

        let file_sut = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |_| {});

        assert_that!(file_sut, is_err);
        assert_that!(
            file_sut.err().unwrap(), eq SharedMemoryDirectoryCreateFileError::DoesExist
        );
    }

    #[test]
    fn can_recreate_file_after_removal() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let file_name = generate_name();
        let file = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |_| {})
            .unwrap();

        drop(file);

        let file_sut = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |_| {});

        assert_that!(file_sut, is_ok);
    }

    #[test]
    fn list_files_work() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let mut file_names = vec![];
        for _ in 0..sut.file_capacity() {
            let file_name = generate_name();
            file_names.push(file_name.clone());
            let _file = sut
                .new_file(Layout::new::<u8>())
                .unwrap()
                .is_persistent(true)
                .create(&file_name, |_| {})
                .unwrap();
        }

        let list = sut.list_files();
        for file_name in file_names {
            let mut has_found = false;
            for file in &list {
                if file.name() == file_name {
                    has_found = true;
                    break;
                }
            }

            assert_that!(has_found, eq true);
        }
    }

    #[test]
    fn new_and_open_many_files_with_different_shm_instances() {
        let name = generate_name();

        let sut_1 = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let _sut_2 = SharedMemoryDirectoryCreator::new(&name)
            .open::<MgmtShm, Allocator, DataShm>()
            .unwrap();

        let mut files = vec![];

        let mut counter = 0u8;
        for _ in 0..sut_1.file_capacity() {
            let file_name = generate_name();
            files.push(file_name.clone());

            assert_that!(sut_1.does_file_exist(&file_name), eq false);

            let mut file = sut_1
                .new_file(Layout::new::<u64>())
                .unwrap()
                .is_persistent(true)
                .create(&file_name, |data_ptr| {
                    assert_that!(data_ptr, len 8);
                    data_ptr[0] = counter;
                    data_ptr[1] = counter;
                    data_ptr[2] = counter;
                    data_ptr[3] = counter;
                })
                .unwrap();

            assert_that!(file.name(), eq file_name);
            assert_that!(file.is_persistent(), eq true);

            file.content_mut()[0] = 255;
            file.content_mut()[2] = 0;

            drop(file);

            assert_that!(sut_1.does_file_exist(&file_name), eq true);

            counter = (counter + 1) % 254;
        }

        let mut counter = 0u8;
        for file_name in files {
            let file = sut_1.open_file(&file_name).unwrap();

            assert_that!(file.name(), eq file_name);
            assert_that!(file.is_persistent(), eq true);
            assert_that!(file.content()[0], eq 255);
            assert_that!(file.content()[1], eq counter);
            assert_that!(file.content()[2], eq 0);
            assert_that!(file.content()[3], eq counter);

            counter = (counter + 1) % 254;
        }
    }

    #[test]
    fn remove_makes_file_available_even_when_opened() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let file_name = generate_name();
        let file = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |data_ptr| {
                data_ptr[0] = 123;
            })
            .unwrap();

        assert_that!(sut.does_file_exist(&file_name), eq true);
        assert_that!(sut.remove_file(&file_name), eq true);
        assert_that!(sut.remove_file(&file_name), eq false);
        assert_that!(sut.does_file_exist(&file_name), eq false);

        let file_2 = sut.open_file(&file_name);

        assert_that!(file_2, is_none);

        let _file_3 = sut
            .new_file(Layout::new::<u8>())
            .unwrap()
            .create(&file_name, |data_ptr| {
                data_ptr[0] = 0;
            })
            .unwrap();

        assert_that!(file.content()[0], eq 123);
    }

    #[test]
    fn create_reserves_file_name() {
        let name = generate_name();

        let sut = SharedMemoryDirectoryCreator::new(&name)
            .size(1024 * 1024)
            .create::<MgmtShm, Allocator, DataShm>(
                &<Allocator as ShmAllocator>::Configuration::default(),
            )
            .unwrap();

        let file_name = generate_name();
        let in_create = Barrier::new(2);
        let finish_create = Barrier::new(2);

        std::thread::scope(|s| {
            let _t1 = s.spawn(|| {
                let sut_2 = SharedMemoryDirectoryCreator::new(&name)
                    .open::<MgmtShm, Allocator, DataShm>()
                    .unwrap();

                let _file = sut_2
                    .new_file(Layout::new::<u8>())
                    .unwrap()
                    .create(&file_name, |data_ptr| {
                        in_create.wait();
                        data_ptr[0] = 123;
                        finish_create.wait();
                    })
                    .unwrap();
            });

            in_create.wait();

            let does_exist_result = sut.does_file_exist(&file_name);
            let remove_result = sut.remove_file(&file_name);

            let file_2 =
                sut.new_file(Layout::new::<u8>())
                    .unwrap()
                    .create(&file_name, |data_ptr| {
                        in_create.wait();
                        data_ptr[0] = 123;
                        finish_create.wait();
                    });

            finish_create.wait();

            assert_that!(file_2, is_err);
            assert_that!(file_2.err().unwrap(), eq SharedMemoryDirectoryCreateFileError::BeingCreated);
            assert_that!(does_exist_result, eq false);
            assert_that!(remove_result, eq false);
        });
    }
}
