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
    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_elementary::math::ToB64;
    use iceoryx2_bb_posix::creation_mode::CreationMode;
    use iceoryx2_bb_posix::shared_memory::SharedMemoryBuilder;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::dynamic_storage::*;
    use iceoryx2_cal::named_concept::*;

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"test_").unwrap();
        file.push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
            .unwrap();
        file
    }

    #[derive(Debug)]
    struct TestData {}

    unsafe impl Send for TestData {}
    unsafe impl Sync for TestData {}

    #[test]
    fn version_check_works() {
        type Sut = iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage<TestData>;
        let storage_name = generate_name();
        let file_name = <Sut as NamedConceptMgmt>::Configuration::default()
            .path_for(&storage_name)
            .file_name();

        let raw_shm = SharedMemoryBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(1234)
            .has_ownership(true)
            .create()
            .unwrap();

        unsafe {
            *(raw_shm.base_address().as_ptr() as *mut u64) = u64::MAX;
        }

        let sut = <Sut as DynamicStorage<TestData>>::Builder::new(&storage_name).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq DynamicStorageOpenError::VersionMismatch);
    }
}
