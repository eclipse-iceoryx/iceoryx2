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

mod dynamic_storage_posix_shared_memory {
    use core::time::Duration;
    use iceoryx2_bb_posix::creation_mode::CreationMode;
    use iceoryx2_bb_posix::permission::Permission;
    use iceoryx2_bb_posix::shared_memory::SharedMemoryBuilder;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::dynamic_storage::*;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;

    const TIMEOUT: Duration = Duration::from_millis(100);

    #[derive(Debug)]
    struct TestData {}

    unsafe impl Send for TestData {}
    unsafe impl Sync for TestData {}

    #[test]
    fn version_check_works() {
        type Sut = iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<TestData>;
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let file_name = config.path_for(&storage_name).file_name();

        let raw_shm = SharedMemoryBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(1234)
            .has_ownership(true)
            .create()
            .unwrap();

        unsafe {
            *(raw_shm.base_address().as_ptr() as *mut u64) = u64::MAX;
        }

        let sut = <Sut as DynamicStorage<TestData>>::Builder::new(&storage_name)
            .config(&config)
            .open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::VersionMismatch);
    }

    #[test]
    fn write_only_segment_is_not_initialized() {
        type Sut = iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<TestData>;
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let file_name = config.path_for(&storage_name).file_name();

        let _raw_shm = SharedMemoryBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(1234)
            .has_ownership(true)
            .permission(Permission::OWNER_WRITE)
            .create()
            .unwrap();

        let sut = <Sut as DynamicStorage<TestData>>::Builder::new(&storage_name)
            .config(&config)
            .open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::InitializationNotYetFinalized);
    }

    #[test]
    fn waiting_for_initialization_works() {
        type Sut = iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<TestData>;
        let storage_name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let file_name = config.path_for(&storage_name).file_name();

        let _raw_shm = SharedMemoryBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(1234)
            .has_ownership(true)
            .permission(Permission::OWNER_WRITE)
            .create()
            .unwrap();

        let start = std::time::SystemTime::now();
        let sut = <Sut as DynamicStorage<TestData>>::Builder::new(&storage_name)
            .timeout(TIMEOUT)
            .config(&config)
            .open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::InitializationNotYetFinalized);
        assert_that!(start.elapsed().unwrap(), ge TIMEOUT);
    }
}
