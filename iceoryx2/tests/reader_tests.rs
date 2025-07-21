// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
//
#[generic_tests::define]
mod reader {
    use iceoryx2::port::reader::*;
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use std::collections::HashSet;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn id_is_unique<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_READERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .max_readers(MAX_READERS)
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let mut readers = vec![];
        let mut reader_id_set = HashSet::new();

        for _ in 0..MAX_READERS {
            let reader = sut.reader_builder().create().unwrap();
            assert_that!(reader_id_set.insert(reader.id()), eq true);
            readers.push(reader);
        }
    }

    #[test]
    fn handle_can_be_acquired_for_existing_key_value_pair<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u64>(&0);
        assert_that!(reader_handle, is_ok);
    }

    #[test]
    fn handle_cannot_be_acquired_for_non_existing_key<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<u64>(&9);
        assert_that!(reader_handle, is_err);
        assert_that!(
            reader_handle.err().unwrap(),
            eq ReaderHandleError::EntryDoesNotExist
        );
    }

    #[test]
    fn handle_cannot_be_acquired_for_wrong_value_type<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<u64>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let reader_handle = reader.entry::<i64>(&0);
        assert_that!(reader_handle, is_err);
        assert_that!(
            reader_handle.err().unwrap(),
            eq ReaderHandleError::EntryDoesNotExist
        );
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    #[instantiate_tests(<iceoryx2::service::ipc_threadsafe::Service>)]
    mod ipc_threadsafe {}

    #[instantiate_tests(<iceoryx2::service::local_threadsafe::Service>)]
    mod local_threadsafe {}
}
