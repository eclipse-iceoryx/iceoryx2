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
mod active_request {
    use iceoryx2::port::client::Client;
    use iceoryx2::port::server::Server;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2::{
        node::{Node, NodeBuilder},
        prelude::ZeroCopySend,
        service::Service,
    };
    use iceoryx2_bb_testing::assert_that;

    struct TestFixture<Sut: Service> {
        _node: Node<Sut>,
        service: PortFactory<Sut, u64, u64, u64, ()>,
        client: Client<Sut, u64, u64, u64, ()>,
        server: Server<Sut, u64, u64, u64, ()>,
    }

    impl<Sut: Service> TestFixture<Sut> {
        fn new() -> Self {
            let config = generate_isolated_config();
            let service_name = generate_service_name();
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let service = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .request_user_header::<u64>()
                .max_servers(1)
                .max_clients(2)
                .create()
                .unwrap();
            let client = service.client_builder().create().unwrap();
            let server = service.server_builder().create().unwrap();

            Self {
                _node: node,
                service,
                client,
                server,
            }
        }
    }

    #[test]
    fn is_connected_until_pending_response_is_dropped<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let pending_response = test.client.send_copy(123).unwrap();

        let sut = test.server.receive().unwrap().unwrap();
        assert_that!(sut.is_connected(), eq true);

        drop(pending_response);
        assert_that!(sut.is_connected(), eq false);
    }

    #[test]
    fn is_connected_until_pending_response_is_dropped_multiple_connections<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let pending_response_1 = test.client.send_copy(123).unwrap();
        let pending_response_2 = test.client.send_copy(123).unwrap();
        let pending_response_3 = test.client.send_copy(123).unwrap();

        let sut_1 = test.server.receive().unwrap().unwrap();
        let sut_2 = test.server.receive().unwrap().unwrap();
        let sut_3 = test.server.receive().unwrap().unwrap();

        assert_that!(sut_1.is_connected(), eq true);
        assert_that!(sut_2.is_connected(), eq true);
        assert_that!(sut_3.is_connected(), eq true);

        drop(pending_response_2);

        assert_that!(sut_1.is_connected(), eq true);
        assert_that!(sut_2.is_connected(), eq false);
        assert_that!(sut_3.is_connected(), eq true);

        drop(pending_response_3);

        assert_that!(sut_1.is_connected(), eq true);
        assert_that!(sut_2.is_connected(), eq false);
        assert_that!(sut_3.is_connected(), eq false);

        drop(pending_response_1);

        assert_that!(sut_1.is_connected(), eq false);
        assert_that!(sut_2.is_connected(), eq false);
        assert_that!(sut_3.is_connected(), eq false);
    }

    #[test]
    fn keeps_being_connected_when_client_goes_out_of_scope<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let client2 = test.service.client_builder().create().unwrap();
        let pending_response = client2.send_copy(123).unwrap();

        let sut = test.server.receive().unwrap().unwrap();
        assert_that!(sut.is_connected(), eq true);

        drop(client2);
        assert_that!(sut.is_connected(), eq true);

        drop(pending_response);
        assert_that!(sut.is_connected(), eq false);
    }

    #[test]
    fn drop_closes_connection_to_pending_response<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let pending_response = test.client.send_copy(123).unwrap();

        let sut = test.server.receive().unwrap().unwrap();
        assert_that!(pending_response.is_connected(), eq true);

        drop(sut);
        assert_that!(pending_response.is_connected(), eq false);
    }

    #[test]
    fn loan_uninit_and_send_works<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let pending_response = test.client.send_copy(123).unwrap();

        let sut = test.server.receive().unwrap().unwrap();
        let loan = sut.loan_uninit().unwrap();

        assert_that!(pending_response.has_response(), eq false);
        loan.write_payload(456).send().unwrap();

        assert_that!(pending_response.has_response(), eq true);
    }

    #[test]
    fn send_copy_works<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let pending_response = test.client.send_copy(123).unwrap();

        let sut = test.server.receive().unwrap().unwrap();

        assert_that!(pending_response.has_response(), eq false);
        sut.send_copy(456).unwrap();

        assert_that!(pending_response.has_response(), eq true);
    }

    #[test]
    fn loan_send_works<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let pending_response = test.client.send_copy(123).unwrap();

        let sut = test.server.receive().unwrap().unwrap();
        let mut loan = sut.loan().unwrap();

        assert_that!(pending_response.has_response(), eq false);
        *loan = 456;
        loan.send().unwrap();

        assert_that!(pending_response.has_response(), eq true);
    }

    #[derive(Debug, ZeroCopySend, Eq, PartialEq)]
    #[repr(C)]
    struct CustomUserHeader<const A: u32, const B: u64> {
        data_a: u32,
        data_b: u64,
    }

    impl<const A: u32, const B: u64> Default for CustomUserHeader<A, B> {
        fn default() -> Self {
            Self {
                data_a: A,
                data_b: B,
            }
        }
    }

    #[test]
    fn loaned_response_has_default_constructed_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<44491, 55592>;
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .response_user_header::<UserHeader>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let server = service.server_builder().create().unwrap();

        let _pending_response = client.send_copy(123).unwrap();

        let active_request = server.receive().unwrap().unwrap();
        let sut = active_request.loan().unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn loaned_uninitialized_response_has_default_constructed_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<1144491, 1155592>;
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .response_user_header::<UserHeader>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let server = service.server_builder().create().unwrap();

        let _pending_response = client.send_copy(123).unwrap();

        let active_request = server.receive().unwrap().unwrap();
        let sut = active_request.loan_uninit().unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn loaned_slice_response_has_default_constructed_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<474491, 575592>;
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, [u64]>()
            .response_user_header::<UserHeader>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let server = service.server_builder().create().unwrap();

        let _pending_response = client.send_copy(123).unwrap();

        let active_request = server.receive().unwrap().unwrap();
        let sut = active_request.loan_slice(1).unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn loaned_uninitialized_slice_response_has_default_constructed_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<47774491, 57577592>;
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, [u64]>()
            .response_user_header::<UserHeader>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let server = service.server_builder().create().unwrap();

        let _pending_response = client.send_copy(123).unwrap();

        let active_request = server.receive().unwrap().unwrap();
        let sut = active_request.loan_slice_uninit(1).unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn origin_is_correctly_set<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let _pending_response = test.client.send_copy(123).unwrap();
        let sut = test.server.receive().unwrap().unwrap();

        assert_that!(sut.origin(), eq test.client.id());
        assert_that!(sut.header().client_id(), eq test.client.id());
    }

    #[test]
    fn payload_and_user_header_are_correctly_set<Sut: Service>() {
        const USER_HEADER: u64 = 910283129;
        const PAYLOAD: u64 = 125894612937;
        let test = TestFixture::<Sut>::new();
        let mut request = test.client.loan().unwrap();
        *request.user_header_mut() = USER_HEADER;
        *request.payload_mut() = PAYLOAD;
        let _pending_response = request.send().unwrap();

        let sut = test.server.receive().unwrap().unwrap();

        assert_that!(*sut.user_header(), eq USER_HEADER);
        assert_that!(*sut.payload(), eq PAYLOAD);
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
