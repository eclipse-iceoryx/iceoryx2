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

mod zero_copy_connection_posix_shared_memory_tests {
    use core::time::Duration;
    use iceoryx2_bb_elementary::math::ToB64;
    use iceoryx2_bb_posix::creation_mode::CreationMode;
    use iceoryx2_bb_posix::permission::Permission;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::zero_copy_connection::*;

    const TIMEOUT: Duration = Duration::from_millis(100);

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"test_").unwrap();
        file.push_bytes(UniqueSystemId::new().unwrap().value().to_b64().as_bytes())
            .unwrap();
        file
    }

    #[test]
    fn waiting_for_initialization_works() {
        type Sut = iceoryx2_cal::zero_copy_connection::posix_shared_memory::Connection;
        let storage_name = generate_name();
        let file_name = <Sut as NamedConceptMgmt>::Configuration::default()
            .path_for(&storage_name)
            .file_name();

        let _raw_shm = iceoryx2_bb_posix::shared_memory::SharedMemoryBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .size(4096)
            .has_ownership(true)
            .permission(Permission::OWNER_WRITE)
            .create()
            .unwrap();

        let start = std::time::SystemTime::now();
        let sut = <Sut as ZeroCopyConnection>::Builder::new(&storage_name)
            .timeout(TIMEOUT)
            .number_of_samples_per_segment(1)
            .receiver_max_borrowed_samples_per_channel(1)
            .create_sender();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq ZeroCopyCreationError::InitializationNotYetFinalized);
        assert_that!(start.elapsed().unwrap(), ge TIMEOUT);
    }
}
