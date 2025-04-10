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
    use std::collections::HashSet;

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::port::client::Client;
    use iceoryx2::port::server::Server;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

    #[derive(Clone, Copy)]
    struct Args {
        number_of_clients: usize,
        number_of_servers: usize,
        response_buffer_size: usize,
        request_overflow: bool,
        response_overflow: bool,
        client_unable_to_deliver_strategy: UnableToDeliverStrategy,
        server_unable_to_deliver_strategy: UnableToDeliverStrategy,
    }

    impl Default for Args {
        fn default() -> Self {
            Self {
                number_of_clients: 1,
                number_of_servers: 1,
                response_buffer_size: 1,
                request_overflow: true,
                response_overflow: true,
                client_unable_to_deliver_strategy: UnableToDeliverStrategy::DiscardSample,
                server_unable_to_deliver_strategy: UnableToDeliverStrategy::DiscardSample,
            }
        }
    }

    struct TestFixture<Sut: Service> {
        _node: Node<Sut>,
        service: PortFactory<Sut, usize, usize, usize, usize>,
        clients: Vec<Client<Sut, usize, usize, usize, usize>>,
        servers: Vec<Server<Sut, usize, usize, usize, usize>>,
    }

    impl<Sut: Service> TestFixture<Sut> {
        fn new(args: Args) -> Self {
            let config = generate_isolated_config();
            let service_name = generate_service_name();
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let service = node
                .service_builder(&service_name)
                .request_response::<usize, usize>()
                .request_user_header::<usize>()
                .response_user_header::<usize>()
                .enable_safe_overflow_for_requests(args.request_overflow)
                .enable_safe_overflow_for_responses(args.response_overflow)
                .max_response_buffer_size(args.response_buffer_size)
                .max_servers(args.number_of_servers)
                .max_clients(args.number_of_clients)
                .create()
                .unwrap();

            let mut servers = vec![];
            for _ in 0..args.number_of_servers {
                servers.push(
                    service
                        .server_builder()
                        .unable_to_deliver_strategy(args.server_unable_to_deliver_strategy)
                        .create()
                        .unwrap(),
                );
            }

            let mut clients = vec![];
            for _ in 0..args.number_of_clients {
                clients.push(
                    service
                        .client_builder()
                        .unable_to_deliver_strategy(args.client_unable_to_deliver_strategy)
                        .create()
                        .unwrap(),
                );
            }

            Self {
                _node: node,
                service,
                clients,
                servers,
            }
        }
    }

    #[test]
    fn response_buffer_size_of_client_is_set_correctly<Sut: Service>() {
        let test_args = Args {
            response_buffer_size: 7,
            ..Default::default()
        };

        const REQUEST_HEADER: usize = 918239;
        const RESPONSE_HEADER: usize = 438921412;
        const PAYLOAD: usize = 12;

        let test = TestFixture::<Sut>::new(test_args);

        let mut request = test.clients[0].loan().unwrap();
        *request.user_header_mut() = REQUEST_HEADER;
        *request.payload_mut() = PAYLOAD;

        let pending_response = request.send().unwrap();

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq PAYLOAD);
        assert_that!(*active_request.user_header(), eq REQUEST_HEADER);

        for n in 0..test_args.response_buffer_size {
            let mut response = active_request.loan().unwrap();
            *response.user_header_mut() = RESPONSE_HEADER + n;
            *response.payload_mut() = PAYLOAD + n;
            assert_that!(response.send(), is_ok);
        }

        for n in 0..test_args.response_buffer_size {
            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response.payload(), eq PAYLOAD + n);
            assert_that!(*response.user_header(), eq RESPONSE_HEADER + n);
        }
    }

    #[test]
    fn response_buffer_size_with_overflow_works<Sut: Service>() {
        let test_args = Args {
            response_buffer_size: 5,
            response_overflow: true,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let pending_response = test.clients[0].send_copy(0).unwrap();
        let active_request = test.servers[0].receive().unwrap().unwrap();

        for n in 0..test_args.response_buffer_size * 2 {
            assert_that!(active_request.send_copy(n), is_ok);
        }

        for n in 0..test_args.response_buffer_size {
            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response.payload(), eq test_args.response_buffer_size + n);
        }
    }

    #[test]
    fn response_buffer_size_with_non_overflow_works<Sut: Service>() {
        let test_args = Args {
            response_buffer_size: 9,
            response_overflow: false,
            server_unable_to_deliver_strategy: UnableToDeliverStrategy::DiscardSample,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let pending_response = test.clients[0].send_copy(0).unwrap();
        let active_request = test.servers[0].receive().unwrap().unwrap();

        for n in 0..test_args.response_buffer_size * 2 {
            assert_that!(active_request.send_copy(n), is_ok);
        }

        for n in 0..test_args.response_buffer_size {
            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response.payload(), eq n);
        }
    }

    #[test]
    fn responses_are_delivered_only_to_the_client_with_the_request<Sut: Service>() {
        let test_args = Args {
            number_of_clients: 4,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let mut pending_responses = vec![];
        for (n, client) in test.clients.iter().enumerate() {
            pending_responses.push(client.send_copy(4 * n + 3).unwrap());
        }

        let mut active_requests = vec![];
        for _ in 0..test_args.number_of_clients {
            active_requests.push(test.servers[0].receive().unwrap().unwrap());
        }

        for active_request in active_requests {
            assert_that!(
                active_request.send_copy(*active_request.payload() + 5),
                is_ok
            );
        }

        for pending_response in pending_responses {
            let n = *pending_response.payload();
            assert_that!(*pending_response.receive().unwrap().unwrap(), eq n + 5);
            assert_that!(pending_response.receive().unwrap(), is_none);
        }
    }

    #[test]
    fn responses_are_delivered_from_all_servers<Sut: Service>() {
        let test_args = Args {
            number_of_servers: 3,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let pending_responses = test.clients[0].send_copy(0).unwrap();

        let mut server_ids = HashSet::new();
        let mut active_requests = vec![];
        for n in 0..test_args.number_of_servers {
            active_requests.push(test.servers[n].receive().unwrap().unwrap());
            server_ids.insert(test.servers[n].id());
        }

        for active_request in active_requests {
            assert_that!(active_request.send_copy(0), is_ok);
        }

        for _ in 0..test_args.number_of_servers {
            let response = pending_responses.receive().unwrap().unwrap();
            assert_that!(server_ids, contains response.origin());
            server_ids.remove(&response.origin());
        }
    }

    #[test]
    fn responses_from_previous_requests_are_filtered_out<Sut: Service>() {
        const ITERATIONS: usize = 19;
        let test_args = Args {
            response_buffer_size: 9,
            response_overflow: true,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        for i in 0..ITERATIONS {
            let pending_response = test.clients[0].send_copy(0).unwrap();
            let active_request = test.servers[0].receive().unwrap().unwrap();

            // fill full buffer
            for n in 0..test_args.response_buffer_size {
                assert_that!(active_request.send_copy(n * i), is_ok);
            }

            // just grab half of the buffer, the remainders must be cleaned up
            for n in 0..test_args.response_buffer_size / 2 {
                let response = pending_response.receive().unwrap().unwrap();
                assert_that!(*response, eq n * i);
            }
        }
    }

    #[test]
    fn communication_with_max_clients_and_servers_works<Sut: Service>() {
        const MAX_CLIENTS: usize = 4;
        const MAX_SERVERS: usize = 4;
        const MAX_ACTIVE_REQUESTS: usize = 2;

        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        for max_clients in 1..MAX_CLIENTS {
            for max_servers in 1..MAX_SERVERS {
                let sut = node
                    .service_builder(&service_name)
                    .request_response::<u64, u64>()
                    .max_clients(max_clients)
                    .max_servers(max_servers)
                    .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
                    .create()
                    .unwrap();

                let mut clients = vec![];
                let mut servers = vec![];

                for _ in 0..max_clients {
                    clients.push(sut.client_builder().create().unwrap());
                }

                for _ in 0..max_servers {
                    servers.push(sut.server_builder().create().unwrap());
                }

                for n in 0..MAX_ACTIVE_REQUESTS {
                    let mut pending_responses = vec![];
                    for client in &clients {
                        pending_responses.push(client.send_copy(4 * n as u64 + 3).unwrap());
                    }

                    for server in &servers {
                        let received_request = server.receive().unwrap().unwrap();
                        assert_that!(*received_request, eq 4 * n as u64 + 3);
                    }
                }
            }
        }
    }

    #[test]
    fn dropping_service_keeps_established_communication<Sut: Service>() {
        let service_name = generate_service_name();
        let config = generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let server = sut.server_builder().create().unwrap();
        let client = sut.client_builder().create().unwrap();

        drop(sut);

        assert_that!(client.send_copy(8182982), is_ok);
        assert_that!(*server.receive().unwrap().unwrap(), eq 8182982);
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
