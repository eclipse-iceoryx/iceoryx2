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
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
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
        type ValueType = u64;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<ValueType>(0, 0)
            .create()
            .unwrap();

        let reader = sut.reader_builder().create().unwrap();
        let entry_handle = reader.entry::<ValueType>(&0);
        assert_that!(entry_handle, is_ok);
        assert_that!(entry_handle.unwrap().get(), eq 0);
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
        let entry_handle = reader.entry::<u64>(&9);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
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
        let entry_handle = reader.entry::<i64>(&0);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
        );
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn handle_can_be_acquired_for_existing_key_value_pair_with_custom_key_type<Sut: Service>() {
        type ValueType = u64;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add::<ValueType>(0, 0)
            .create()
            .unwrap();
        let reader = sut.reader_builder().create().unwrap();

        let type_details = TypeDetail::new::<ValueType>(TypeVariant::FixedSize);
        let entry_handle = reader.__internal_entry(&0, &type_details);
        assert_that!(entry_handle, is_ok);
        let mut read_value: ValueType = 9;
        let read_value_ptr: *mut ValueType = &mut read_value;
        unsafe {
            entry_handle.unwrap().get(
                read_value_ptr as *mut u8,
                size_of::<ValueType>(),
                align_of::<ValueType>(),
            );
        }
        assert_that!(read_value, eq 0);
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn handle_cannot_be_acquired_for_non_existing_key_with_custom_key_type<Sut: Service>() {
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

        let type_details = TypeDetail::new::<u64>(TypeVariant::FixedSize);
        let entry_handle = reader.__internal_entry(&9, &type_details);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
        );
    }

    // TODO [#817] replace u64 with CustomKeyMarker
    #[test]
    fn handle_cannot_be_acquired_for_wrong_value_type_with_custom_key_type<Sut: Service>() {
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

        let type_details = TypeDetail::new::<i64>(TypeVariant::FixedSize);
        let entry_handle = reader.__internal_entry(&0, &type_details);
        assert_that!(entry_handle, is_err);
        assert_that!(
            entry_handle.err().unwrap(),
            eq EntryHandleError::EntryDoesNotExist
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
