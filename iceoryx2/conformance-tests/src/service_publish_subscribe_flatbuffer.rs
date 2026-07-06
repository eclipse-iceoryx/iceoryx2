// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

extern crate alloc;

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod service_publish_subscribe_flatbuffer {
    use iceoryx2::service::builder::publish_subscribe::{
        PublishSubscribeCreateError, PublishSubscribeOpenError,
    };
    use iceoryx2::service::{Service, marker::Flatbuffer};
    use iceoryx2_bb_posix::file::{CreationMode, File, FileBuilder};
    use iceoryx2_bb_posix::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::static_storage::StaticStorage;
    use iceoryx2_testing::*;

    const SCHEMA: &str = "
        namespace Example;

        table Entry {
            data_1: int32;
            data_2: uint64;
        }

        table UnboundedData {
            title: string;
            entries: [Entry];
        }

        root_type UnboundedData;
    ";

    const ALT_SCHEMA: &str = "
        namespace Example;

        table BoundedData {
            data_1: int32;
        }

        root_type BoundedData;
    ";

    fn create_schema_file(schema: &str) -> File {
        let schema_file = generate_file_path();
        let mut file = FileBuilder::new(&schema_file)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        file.acquire_ownership();
        file.write(schema.as_bytes()).unwrap();
        file
    }

    #[conformance_test]
    pub fn create_fails_when_no_schema_file_is_available<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .create();

        assert_that!(sut.err(), eq Some(PublishSubscribeCreateError::UnableToAcquireTypeDefinition));
    }

    #[conformance_test]
    pub fn create_succeeds_with_schema_file<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let schema_file = create_schema_file(SCHEMA);

        let service_name = generate_service_name();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(schema_file.path().unwrap())
            .create();

        assert_that!(sut, is_ok);
    }

    #[conformance_test]
    pub fn open_fails_when_no_schema_file_is_available<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        let schema_file = create_schema_file(SCHEMA);

        let _sut_create = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(schema_file.path().unwrap())
            .create();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .open();

        assert_that!(sut.err(), eq Some(PublishSubscribeOpenError::UnableToAcquireTypeDefinition));
    }

    #[conformance_test]
    pub fn open_fails_when_no_schema_is_not_the_same<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        let schema_file = create_schema_file(SCHEMA);
        let alt_schema_file = create_schema_file(ALT_SCHEMA);

        let _sut_create = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(schema_file.path().unwrap())
            .create();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(alt_schema_file.path().unwrap())
            .open();

        assert_that!(sut.err(), eq Some(PublishSubscribeOpenError::IncompatibleTypes));
    }

    #[conformance_test]
    pub fn open_succeeds_when_schema_content_is_identical<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        let schema_file = create_schema_file(SCHEMA);
        let alt_schema_file = create_schema_file(SCHEMA);

        let _sut_create = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(schema_file.path().unwrap())
            .create();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(alt_schema_file.path().unwrap())
            .open();

        assert_that!(sut, is_ok);
    }

    #[conformance_test]
    pub fn service_schema_is_identical_to_origin<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        let schema_file = create_schema_file(SCHEMA);

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<Flatbuffer<u64>>()
            .flatbuffer_schema_path(schema_file.path().unwrap())
            .create()
            .unwrap();

        let type_definition = sut.type_definition().unwrap();

        assert_that!(type_definition.len(), eq SCHEMA.len() as u64);
        let mut buffer = vec![0u8; type_definition.len() as usize];
        type_definition.read(&mut buffer).unwrap();

        assert_that!(SCHEMA.as_bytes(), eq buffer.as_slice());
    }
}
