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
mod server {
    use iceoryx2::port::ReceiveError;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

    fn create_node<Sut: Service>() -> Node<Sut> {
        let config = generate_isolated_config();
        NodeBuilder::new().config(&config).create::<Sut>().unwrap()
    }

    fn create_node_and_service<Sut: Service>() -> (Node<Sut>, PortFactory<Sut, u64, (), u64, ()>) {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        (node, service)
    }

    #[test]
    fn disconnected_server_does_not_block_new_servers<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_servers(1)
            .create()
            .unwrap();

        let sut = service.server_builder().create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut = service.server_builder().create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn receiving_requests_works_with_server_created_first<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();
        let sut = service.server_builder().create().unwrap();
        let client = service.client_builder().create().unwrap();

        assert_that!(sut.has_requests(), eq Ok(false));
        assert_that!(client.send_copy(1234), is_ok);
        assert_that!(sut.has_requests(), eq Ok(true));

        let active_request = sut.receive().unwrap().unwrap();
        assert_that!(*active_request, eq 1234);
    }

    #[test]
    fn receiving_requests_works_with_client_created_first<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();
        let client = service.client_builder().create().unwrap();
        let sut = service.server_builder().create().unwrap();

        assert_that!(sut.has_requests(), eq Ok(false));
        assert_that!(client.send_copy(5678), is_ok);
        assert_that!(sut.has_requests(), eq Ok(true));

        let active_request = sut.receive().unwrap().unwrap();
        assert_that!(*active_request, eq 5678);
    }

    #[test]
    fn requests_of_a_disconnected_client_are_not_received<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();
        let client = service.client_builder().create().unwrap();
        let sut = service.server_builder().create().unwrap();

        assert_that!(sut.has_requests(), eq Ok(false));
        assert_that!(client.send_copy(5678), is_ok);
        assert_that!(sut.has_requests(), eq Ok(true));
        drop(client);
        assert_that!(sut.has_requests(), eq Ok(false));

        let active_request = sut.receive().unwrap();
        assert_that!(active_request, is_none);
    }

    #[test]
    fn requests_are_not_delivered_to_late_joiners<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();
        let client = service.client_builder().create().unwrap();

        assert_that!(client.send_copy(5678), is_ok);

        let sut = service.server_builder().create().unwrap();
        assert_that!(sut.has_requests(), eq Ok(false));

        let active_request = sut.receive().unwrap();
        assert_that!(active_request, is_none);
    }

    fn server_can_hold_specified_amount_of_active_requests<Sut: Service>(
        max_active_requests: usize,
        max_clients: usize,
    ) {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(max_clients)
            .max_servers(1)
            .max_active_requests_per_client(max_active_requests)
            .create()
            .unwrap();

        let sut = service.server_builder().create().unwrap();
        let mut clients = vec![];

        for _ in 0..max_clients {
            clients.push(service.client_builder().create().unwrap());
        }

        let mut active_requests = vec![];

        for client in clients {
            for n in 0..max_active_requests {
                assert_that!(client.send_copy(n as u64 * 5 + 7), is_ok);
                let active_request = sut.receive().unwrap().unwrap();
                assert_that!(*active_request, eq n as u64 * 5 + 7);
                active_requests.push(active_request);
            }

            assert_that!(client.send_copy(99), is_ok);
            let active_request = sut.receive();
            assert_that!(active_request.err(), eq Some(ReceiveError::ExceedsMaxBorrows));
        }
    }

    #[test]
    fn server_can_hold_specified_amount_of_active_requests_one_client_one_request<Sut: Service>() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[test]
    fn server_can_hold_specified_amount_of_active_requests_one_client_many_requests<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 9;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[test]
    fn server_can_hold_specified_amount_of_active_requests_many_clients_one_request<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 7;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[test]
    fn server_can_hold_specified_amount_of_active_requests_many_clients_many_requests<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 8;
        const MAX_ACTIVE_REQUESTS: usize = 9;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
