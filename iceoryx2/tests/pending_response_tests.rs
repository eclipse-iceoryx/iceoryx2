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
mod pending_response {
    use iceoryx2::port::client::Client;
    use iceoryx2::port::server::Server;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2::{
        node::{Node, NodeBuilder},
        service::Service,
    };
    use iceoryx2_bb_testing::assert_that;

    struct TestFixture<Sut: Service> {
        _node: Node<Sut>,
        _service: PortFactory<Sut, u64, u64, u64, ()>,
        client: Client<Sut, u64, u64, u64, ()>,
        server_1: Server<Sut, u64, u64, u64, ()>,
        server_2: Server<Sut, u64, u64, u64, ()>,
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
                .max_servers(2)
                .max_clients(2)
                .create()
                .unwrap();
            let client = service.client_builder().create().unwrap();

            Self {
                _node: node,
                client,
                server_1: service.server_builder().create().unwrap(),
                server_2: service.server_builder().create().unwrap(),
                _service: service,
            }
        }
    }

    #[test]
    fn is_connected_until_every_active_request_is_dropped<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let sut = test.client.send_copy(123).unwrap();

        assert_that!(sut.is_connected(), eq true);
        let active_request_1 = test.server_1.receive().unwrap().unwrap();
        let active_request_2 = test.server_2.receive().unwrap().unwrap();
        assert_that!(sut.is_connected(), eq true);

        drop(active_request_1);
        assert_that!(sut.is_connected(), eq true);

        drop(active_request_2);
        assert_that!(sut.is_connected(), eq false);
    }

    #[test]
    fn is_not_connected_when_there_are_no_servers<Sut: Service>() {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let sut = client.send_copy(123).unwrap();

        assert_that!(sut.is_connected(), eq false);
    }

    #[test]
    fn disconnects_on_drop<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let sut = test.client.send_copy(123).unwrap();

        let active_request = test.server_1.receive().unwrap().unwrap();
        assert_that!(active_request.is_connected(), eq true);

        drop(sut);
        assert_that!(active_request.is_connected(), eq false);
    }

    #[test]
    fn payload_header_and_user_header_are_set_correctly<Sut: Service>() {
        const PAYLOAD: u64 = 80917290837;
        const USER_HEADER: u64 = 992101;
        let test = TestFixture::<Sut>::new();
        let mut request = test.client.loan().unwrap();
        *request.payload_mut() = PAYLOAD;
        *request.user_header_mut() = USER_HEADER;
        let sut = request.send().unwrap();

        assert_that!(sut.header().client_id(), eq test.client.id());
        assert_that!(*sut.user_header(), eq USER_HEADER);
        assert_that!(*sut.payload(), eq PAYLOAD);
    }

    #[test]
    fn number_of_server_connections_is_set_correctly<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let sut = test.client.send_copy(9).unwrap();

        assert_that!(sut.number_of_server_connections(), eq 2);
    }

    #[test]
    fn receive_acquires_response<Sut: Service>() {
        let test = TestFixture::<Sut>::new();
        let sut = test.client.send_copy(9).unwrap();

        let active_request = test.server_1.receive().unwrap().unwrap();
        active_request.send_copy(8).unwrap();

        assert_that!(sut.has_response(), eq true);
        assert_that!(sut.receive().unwrap(), is_some);
        assert_that!(sut.has_response(), eq false);
        assert_that!(sut.receive().unwrap(), is_none);
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
