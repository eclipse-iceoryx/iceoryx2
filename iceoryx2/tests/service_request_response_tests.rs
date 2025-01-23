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

#[generic_tests::define]
mod service_request_response {
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::request_response::{
        RequestResponseCreateError, RequestResponseOpenError,
    };
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn open_existing_service_works<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();

        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_non_existing_service_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotExist) );
    }

    #[test]
    fn creating_existing_service_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(sut_create.err(), eq Some(RequestResponseCreateError::AlreadyExists) );
    }

    #[test]
    fn open_or_create_works_with_existing_and_non_existing_services<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_or_create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_or_create();

        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn when_created_service_goes_out_of_scope_the_service_is_removed<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(false));

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(true));

        drop(sut);

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(false));
    }

    #[test]
    fn when_last_opened_service_goes_out_of_scope_the_service_is_removed<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(false));

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        let sut_open_1 = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();

        let sut_open_2 = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(true));

        drop(sut);

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(true));

        drop(sut_open_1);

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(true));

        drop(sut_open_2);

        assert_that!(
            Sut::does_exist(&service_name, &config, MessagingPattern::RequestResponse), eq Ok(false));
    }

    #[test]
    fn opening_service_with_mismatching_request_type_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<i64, u64>()
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleRequestType));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_header::<u64>()
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleRequestType));
    }

    #[test]
    fn opening_service_with_incompatible_request_type_alignment_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_payload_alignment(Alignment::new(512).unwrap())
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleRequestType));
    }

    #[test]
    fn opening_service_with_compatible_request_type_alignment_works<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_payload_alignment(Alignment::new(512).unwrap())
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_payload_alignment(Alignment::new(128).unwrap())
            .open();

        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn opening_service_with_mismatching_response_type_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, i64>()
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleResponseType));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .response_header::<u64>()
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleResponseType));
    }

    #[test]
    fn opening_service_with_incompatible_response_type_alignment_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .response_payload_alignment(Alignment::new(512).unwrap())
            .open();

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleResponseType));
    }

    #[test]
    fn opening_service_with_compatible_response_type_alignment_works<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_payload_alignment(Alignment::new(512).unwrap())
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_payload_alignment(Alignment::new(128).unwrap())
            .open();

        assert_that!(sut_open, is_ok);
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    // todo:
    //   service does exist, list, remove with rpc
}
