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
    use iceoryx2::service::attribute::*;
    use iceoryx2::service::builder::request_response::{
        RequestResponseCreateError, RequestResponseOpenError,
    };
    use iceoryx2::service::port_factory::client::ClientCreateError;
    use iceoryx2::service::port_factory::server::ServerCreateError;
    use iceoryx2::service::static_config::message_type_details::TypeVariant;
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
            .request_user_header::<u64>()
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
            .response_user_header::<u64>()
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
            .response_payload_alignment(Alignment::new(512).unwrap())
            .create();

        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .response_payload_alignment(Alignment::new(128).unwrap())
            .open();

        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn opening_service_with_attributes_and_acquiring_attributes_works<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let attribute_key: AttributeKey = "wanna try this head".try_into().unwrap();
        let attribute_value: AttributeValue = "no its a brainslug".try_into().unwrap();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let _sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create_with_attributes(
                &AttributeSpecifier::new().define(&attribute_key, &attribute_value),
            )
            .unwrap();

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open()
            .unwrap();

        let attributes = sut_open.attributes();
        assert_that!(attributes.len(), eq 1);
        assert_that!(attributes.key_value(&attribute_key, 0), is_some);
        assert_that!(
            attributes.key_value(&attribute_key, 0).unwrap(),
            eq & attribute_value
        );
    }

    #[test]
    fn opening_service_with_incompatible_attributes_fails<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let attribute_key: AttributeKey = "there is a muffin".try_into().unwrap();
        let attribute_value: AttributeValue = "with molten chocolate".try_into().unwrap();
        let attribute_incompatible_key: AttributeKey = "its delicious".try_into().unwrap();
        let attribute_incompatible_value: AttributeValue = "I wanna eat it".try_into().unwrap();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let _sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create_with_attributes(
                &AttributeSpecifier::new().define(&attribute_key, &attribute_value),
            )
            .unwrap();

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_with_attributes(
                &AttributeVerifier::new().require(&attribute_key, &attribute_incompatible_value),
            );

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleAttributes));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_with_attributes(
                &AttributeVerifier::new().require_key(&attribute_incompatible_key),
            );

        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleAttributes));
    }

    #[test]
    fn opening_service_with_compatible_attributes_works<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let attribute_key: AttributeKey = "kermit the brave knight".try_into().unwrap();
        let attribute_value: AttributeValue =
            "rides on hypnotoad into the sunset".try_into().unwrap();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let _sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create_with_attributes(
                &AttributeSpecifier::new().define(&attribute_key, &attribute_value),
            )
            .unwrap();

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_with_attributes(
                &AttributeVerifier::new().require(&attribute_key, &attribute_value),
            );

        assert_that!(sut_open, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_with_attributes(&AttributeVerifier::new().require_key(&attribute_key));

        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn recreate_after_drop_works<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut2 = node
            .service_builder(&service_name)
            .request_response::<f32, f64>()
            .create();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_request_overflow_requirement<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(true)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(false)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleOverflowBehaviorForRequests));
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_response_overflow_requirement<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_responses(true)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_responses(false)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleOverflowBehaviorForResponses));
    }

    #[test]
    fn open_verifies_fire_and_forget_requests_setting_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_fire_and_forget_requests(true)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_fire_and_forget_requests(false)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::IncompatibleBehaviorForFireAndForgetRequests));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_fire_and_forget_requests(true)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_borrowed_responses_per_pending_response_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_borrowed_responses_per_pending_response(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_borrowed_responses_per_pending_response(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedAmountOfBorrowedResponsesPerPendingResponse));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_borrowed_responses_per_pending_response(10)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_active_requests_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedAmountOfActiveRequestsPerClient));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(10)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_response_buffer_size_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_response_buffer_size(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_response_buffer_size(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedResponseBufferSize));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_response_buffer_size(10)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_amount_of_servers_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_servers(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_servers(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedAmountOfServers));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_servers(9)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_amount_of_clients_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedAmountOfClients));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(9)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_amount_of_nodes_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_nodes(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_nodes(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedAmountOfNodes));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_nodes(9)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn open_verifies_max_loaned_requests_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(10)
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(11)
            .open();
        assert_that!(sut_open.err(), eq Some(RequestResponseOpenError::DoesNotSupportRequestedAmountOfClientRequestLoans));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(9)
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn service_builder_adjusts_config_to_sane_values<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(0)
            .max_response_buffer_size(0)
            .max_borrowed_responses_per_pending_response(0)
            .max_servers(0)
            .max_clients(0)
            .max_nodes(0)
            .max_loaned_requests(0)
            .create();
        assert_that!(sut_create, is_ok);
        let sut_create = sut_create.unwrap();

        assert_that!(sut_create.static_config().max_active_requests_per_client(), eq 1);
        assert_that!(sut_create.static_config().max_response_buffer_size(), eq 1);
        assert_that!(sut_create.static_config().max_servers(), eq 1);
        assert_that!(sut_create.static_config().max_clients(), eq 1);
        assert_that!(sut_create.static_config().max_nodes(), eq 1);
        assert_that!(sut_create.static_config().max_borrowed_responses_per_pending_response(), eq 1);
        assert_that!(sut_create.static_config().max_loaned_requests(), eq 1);
    }

    #[test]
    fn service_builder_parameters_override_default_config<Sut: Service>() {
        let service_name = generate_service_name();
        let mut config = generate_isolated_config();
        let rpc_config = &mut config.defaults.request_response;
        rpc_config.enable_safe_overflow_for_requests = true;
        rpc_config.enable_safe_overflow_for_responses = true;
        rpc_config.enable_fire_and_forget_requests = true;
        rpc_config.max_active_requests_per_client = 100;
        rpc_config.max_response_buffer_size = 100;
        rpc_config.max_borrowed_responses_per_pending_response = 100;
        rpc_config.max_servers = 100;
        rpc_config.max_clients = 100;
        rpc_config.max_nodes = 100;
        rpc_config.max_loaned_requests = 100;

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(false)
            .enable_safe_overflow_for_responses(false)
            .enable_fire_and_forget_requests(false)
            .max_active_requests_per_client(1)
            .max_response_buffer_size(6)
            .max_servers(7)
            .max_clients(8)
            .max_nodes(9)
            .max_borrowed_responses_per_pending_response(10)
            .max_loaned_requests(11)
            .create();
        assert_that!(sut_create, is_ok);
        let sut_create = sut_create.unwrap();

        assert_that!(sut_create.static_config().does_support_fire_and_forget_requests(), eq false);
        assert_that!(sut_create.static_config().has_safe_overflow_for_requests(), eq false);
        assert_that!(sut_create.static_config().has_safe_overflow_for_responses(), eq false);
        assert_that!(sut_create.static_config().max_active_requests_per_client(), eq 1);
        assert_that!(sut_create.static_config().max_response_buffer_size(), eq 6);
        assert_that!(sut_create.static_config().max_servers(), eq 7);
        assert_that!(sut_create.static_config().max_clients(), eq 8);
        assert_that!(sut_create.static_config().max_nodes(), eq 9);
        assert_that!(sut_create.static_config().max_borrowed_responses_per_pending_response(), eq 10);
        assert_that!(sut_create.static_config().max_loaned_requests(), eq 11);
    }

    #[test]
    fn service_builder_uses_config_when_user_sets_nothing<Sut: Service>() {
        let service_name = generate_service_name();
        let mut config = generate_isolated_config();
        let rpc_config = &mut config.defaults.request_response;
        rpc_config.enable_safe_overflow_for_requests = true;
        rpc_config.enable_safe_overflow_for_responses = true;
        rpc_config.max_active_requests_per_client = 11;
        rpc_config.max_response_buffer_size = 16;
        rpc_config.max_servers = 17;
        rpc_config.max_clients = 18;
        rpc_config.max_nodes = 19;
        rpc_config.max_borrowed_responses_per_pending_response = 20;
        rpc_config.max_loaned_requests = 21;
        rpc_config.enable_fire_and_forget_requests = true;

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();
        assert_that!(sut_create, is_ok);
        let sut_create = sut_create.unwrap();

        assert_that!(sut_create.static_config().has_safe_overflow_for_requests(), eq true);
        assert_that!(sut_create.static_config().has_safe_overflow_for_responses(), eq true);
        assert_that!(sut_create.static_config().does_support_fire_and_forget_requests(), eq true);
        assert_that!(sut_create.static_config().max_active_requests_per_client(), eq 11);
        assert_that!(sut_create.static_config().max_response_buffer_size(), eq 16);
        assert_that!(sut_create.static_config().max_servers(), eq 17);
        assert_that!(sut_create.static_config().max_clients(), eq 18);
        assert_that!(sut_create.static_config().max_nodes(), eq 19);
        assert_that!(sut_create.static_config().max_borrowed_responses_per_pending_response(), eq 20);
        assert_that!(sut_create.static_config().max_loaned_requests(), eq 21);
    }

    #[test]
    fn opened_service_reads_config_correctly<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(false)
            .enable_safe_overflow_for_responses(false)
            .enable_fire_and_forget_requests(false)
            .max_active_requests_per_client(1)
            .max_response_buffer_size(6)
            .max_servers(7)
            .max_clients(8)
            .max_nodes(9)
            .max_borrowed_responses_per_pending_response(20)
            .max_loaned_requests(21)
            .create()
            .unwrap();

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open()
            .unwrap();

        assert_that!(sut_open.static_config().has_safe_overflow_for_requests(), eq false);
        assert_that!(sut_open.static_config().has_safe_overflow_for_responses(), eq false);
        assert_that!(sut_open.static_config().does_support_fire_and_forget_requests(), eq false);
        assert_that!(sut_open.static_config().max_active_requests_per_client(), eq 1);
        assert_that!(sut_open.static_config().max_response_buffer_size(), eq 6);
        assert_that!(sut_open.static_config().max_servers(), eq 7);
        assert_that!(sut_open.static_config().max_clients(), eq 8);
        assert_that!(sut_open.static_config().max_nodes(), eq 9);
        assert_that!(sut_open.static_config().max_borrowed_responses_per_pending_response(), eq 20);
        assert_that!(sut_open.static_config().max_loaned_requests(), eq 21);
    }

    #[test]
    fn list_finds_created_services<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 12;
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();
            let sut = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .create()
                .unwrap();
            services.push(sut);
            service_names.push(service_name);
        }

        for name in &service_names {
            assert_that!(Sut::does_exist(name, &config, MessagingPattern::RequestResponse).unwrap(), eq true);
        }

        Sut::list(&config, |service| {
            assert_that!(service_names, contains * service.static_details.name());
            service_names.retain(|v| v != service.static_details.name());
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(service_names, len 0);
    }

    #[test]
    fn service_cannot_be_opened_by_more_nodes_than_specified<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();

        let node_1 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let node_3 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_1 = node_1
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_nodes(2)
            .create();
        assert_that!(sut_1, is_ok);

        let sut_2 = node_2
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(sut_2, is_ok);

        let sut_3 = node_3
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(sut_3.err(), eq Some(RequestResponseOpenError::ExceedsMaxNumberOfNodes));
        drop(sut_2);

        let sut_3 = node_3
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(sut_3, is_ok);
    }

    #[test]
    fn server_port_of_dropped_service_blocks_new_service_creation<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let _server = sut.server_builder().create().unwrap();
        drop(sut);

        let result = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();
        assert_that!(result.err(), eq Some(RequestResponseCreateError::AlreadyExists));

        let result = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(result, is_ok);
    }

    #[test]
    fn client_port_of_dropped_service_blocks_new_service_creation<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let _client = sut.client_builder().create().unwrap();
        drop(sut);

        let result = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();
        assert_that!(result.err(), eq Some(RequestResponseCreateError::AlreadyExists));

        let result = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(result, is_ok);
    }

    #[test]
    fn active_request_response_connection_blocks_new_service_creation<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let client = sut.client_builder().create().unwrap();
        let server = sut.server_builder().create().unwrap();

        let _pending_response = client.send_copy(0).unwrap();
        let _active_request = server.receive().unwrap().unwrap();

        drop(sut);
        drop(client);
        drop(server);

        let result = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create();
        assert_that!(result.err(), eq Some(RequestResponseCreateError::AlreadyExists));

        let result = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(result, is_ok);
    }

    #[test]
    fn service_cannot_be_opened_by_more_clients_than_specified<Sut: Service>() {
        const MAX_CLIENTS: usize = 8;
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        for max_clients in 1..MAX_CLIENTS {
            let sut = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .max_clients(max_clients)
                .create()
                .unwrap();

            let mut clients = vec![];

            for _ in 0..max_clients {
                let client = sut.client_builder().create();
                assert_that!(client, is_ok);
                clients.push(client);
            }

            let client = sut.client_builder().create();
            assert_that!(client.err(), eq Some(ClientCreateError::ExceedsMaxSupportedClients));
        }
    }

    #[test]
    fn service_cannot_be_opened_by_more_servers_than_specified<Sut: Service>() {
        const MAX_SERVERS: usize = 8;
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        for max_servers in 1..MAX_SERVERS {
            let sut = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .max_servers(max_servers)
                .create()
                .unwrap();

            let mut servers = vec![];

            for _ in 0..max_servers {
                let server = sut.server_builder().create();
                assert_that!(server, is_ok);
                servers.push(server);
            }

            let server = sut.server_builder().create();
            assert_that!(server.err(), eq Some(ServerCreateError::ExceedsMaxSupportedServers));
        }
    }

    #[test]
    fn type_informations_are_correct<Sut: Service>() {
        type RequestPayload = u128;
        type RequestHeader = f32;
        type ResponsePayload = u8;
        type ResponseHeader = u16;

        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<RequestPayload, ResponsePayload>()
            .request_user_header::<RequestHeader>()
            .response_user_header::<ResponseHeader>()
            .create()
            .unwrap();

        let d = sut.static_config().request_message_type_details();
        assert_that!(d.header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.header.type_name(), eq core::any::type_name::<iceoryx2::service::header::request_response::RequestHeader>());
        assert_that!(d.header.size(), eq core::mem::size_of::<iceoryx2::service::header::request_response::RequestHeader>());
        assert_that!(d.header.alignment(), eq core::mem::align_of::<iceoryx2::service::header::request_response::RequestHeader>());
        assert_that!(d.user_header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.user_header.type_name(), eq core::any::type_name::<RequestHeader>());
        assert_that!(d.user_header.size(), eq core::mem::size_of::<RequestHeader>());
        assert_that!(d.user_header.alignment(), eq core::mem::align_of::<RequestHeader>());
        assert_that!(d.payload.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.payload.type_name(), eq core::any::type_name::<RequestPayload>());
        assert_that!(d.payload.size(), eq core::mem::size_of::<RequestPayload>());
        assert_that!(d.payload.alignment(), eq core::mem::align_of::<RequestPayload>());

        let d = sut.static_config().response_message_type_details();
        assert_that!(d.header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.header.type_name(), eq core::any::type_name::<iceoryx2::service::header::request_response::ResponseHeader>());
        assert_that!(d.header.size(), eq core::mem::size_of::<iceoryx2::service::header::request_response::ResponseHeader>());
        assert_that!(d.header.alignment(), eq core::mem::align_of::<iceoryx2::service::header::request_response::ResponseHeader>());
        assert_that!(d.user_header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.user_header.type_name(), eq core::any::type_name::<ResponseHeader>());
        assert_that!(d.user_header.size(), eq core::mem::size_of::<ResponseHeader>());
        assert_that!(d.user_header.alignment(), eq core::mem::align_of::<ResponseHeader>());
        assert_that!(d.payload.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.payload.type_name(), eq core::any::type_name::<ResponsePayload>());
        assert_that!(d.payload.size(), eq core::mem::size_of::<ResponsePayload>());
        assert_that!(d.payload.alignment(), eq core::mem::align_of::<ResponsePayload>());
    }

    #[test]
    fn custom_request_alignment_cannot_be_smaller_than_type_alignment<Sut: Service>() {
        type RequestPayload = u64;

        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<RequestPayload, u8>()
            .request_payload_alignment(Alignment::new(2).unwrap())
            .create()
            .unwrap();

        assert_that!(sut.static_config().request_message_type_details().payload.alignment(), eq core::mem::align_of::<RequestPayload>());
    }

    #[test]
    fn custom_response_alignment_cannot_be_smaller_than_type_alignment<Sut: Service>() {
        type ResponsePayload = u64;

        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u8, ResponsePayload>()
            .response_payload_alignment(Alignment::new(2).unwrap())
            .create()
            .unwrap();

        assert_that!(sut.static_config().response_message_type_details().payload.alignment(), eq core::mem::align_of::<ResponsePayload>());
    }

    #[test]
    fn create_service_with_request_slice_type_works<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<[u64], u64>()
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open_fail = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(sut_open_fail.err(), eq Some(RequestResponseOpenError::IncompatibleRequestType));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<[u64], u64>()
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn create_service_with_response_slice_type_works<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<u64, [u64]>()
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open_fail = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open();
        assert_that!(sut_open_fail.err(), eq Some(RequestResponseOpenError::IncompatibleResponseType));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<u64, [u64]>()
            .open();
        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn create_service_with_request_and_response_slice_type_works<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .request_response::<[u64], [u64]>()
            .create();
        assert_that!(sut_create, is_ok);

        let sut_open_fail = node
            .service_builder(&service_name)
            .request_response::<[u64], u64>()
            .open();
        assert_that!(sut_open_fail.err(), eq Some(RequestResponseOpenError::IncompatibleResponseType));

        let sut_open_fail = node
            .service_builder(&service_name)
            .request_response::<u64, [u64]>()
            .open();
        assert_that!(sut_open_fail.err(), eq Some(RequestResponseOpenError::IncompatibleRequestType));

        let sut_open = node
            .service_builder(&service_name)
            .request_response::<[u64], [u64]>()
            .open();
        assert_that!(sut_open, is_ok);
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
