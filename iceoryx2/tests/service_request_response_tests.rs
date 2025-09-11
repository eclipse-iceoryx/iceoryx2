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
    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::{PortFactory, *};
    use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::testing;
    use iceoryx2_bb_testing::assert_that;

    #[derive(Clone, Copy)]
    struct Args {
        number_of_active_requests: usize,
        number_of_nodes: usize,
        number_of_clients: usize,
        number_of_servers: usize,
        request_alignment: Alignment,
        response_alignment: Alignment,
        response_buffer_size: usize,
        request_overflow: bool,
        response_overflow: bool,
        enable_fire_and_forget: bool,
        client_unable_to_deliver_strategy: UnableToDeliverStrategy,
        server_unable_to_deliver_strategy: UnableToDeliverStrategy,
    }

    impl Default for Args {
        fn default() -> Self {
            Self {
                number_of_active_requests: 1,
                number_of_nodes: 1,
                number_of_clients: 1,
                number_of_servers: 1,
                request_alignment: Alignment::new(8).unwrap(),
                response_alignment: Alignment::new(8).unwrap(),
                response_buffer_size: 1,
                request_overflow: true,
                response_overflow: true,
                enable_fire_and_forget: false,
                client_unable_to_deliver_strategy: UnableToDeliverStrategy::DiscardSample,
                server_unable_to_deliver_strategy: UnableToDeliverStrategy::DiscardSample,
            }
        }
    }

    struct TestFixture<Sut: Service> {
        node: Node<Sut>,
        service: iceoryx2::service::port_factory::request_response::PortFactory<
            Sut,
            usize,
            usize,
            usize,
            usize,
        >,
        clients: Vec<Client<Sut, usize, usize, usize, usize>>,
        servers: Vec<Server<Sut, usize, usize, usize, usize>>,
    }

    impl<Sut: Service> TestFixture<Sut> {
        fn new(args: Args) -> Self {
            let config = testing::generate_isolated_config();
            let service_name = testing::generate_service_name();
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let service = node
                .service_builder(&service_name)
                .request_response::<usize, usize>()
                .request_user_header::<usize>()
                .response_user_header::<usize>()
                .max_active_requests_per_client(args.number_of_active_requests)
                .max_nodes(args.number_of_nodes)
                .request_payload_alignment(args.request_alignment)
                .response_payload_alignment(args.response_alignment)
                .enable_safe_overflow_for_requests(args.request_overflow)
                .enable_safe_overflow_for_responses(args.response_overflow)
                .max_response_buffer_size(args.response_buffer_size)
                .max_servers(args.number_of_servers)
                .max_clients(args.number_of_clients)
                .enable_fire_and_forget_requests(args.enable_fire_and_forget)
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
                node,
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
        const ITERATIONS: usize = 50;
        let test_args = Args {
            response_buffer_size: 8,
            response_overflow: true,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        for i in 0..ITERATIONS {
            // reset request-response connection in every iteration
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
    fn client_port_ids_are_set_correctly<Sut: Service>() {
        let test_args = Args {
            number_of_clients: 2,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let _pending_response_0 = test.clients[0].send_copy(0).unwrap();
        let _pending_response_1 = test.clients[1].send_copy(1).unwrap();
        let active_request_0 = test.servers[0].receive().unwrap().unwrap();
        let active_request_1 = test.servers[0].receive().unwrap().unwrap();

        let p0 = *active_request_0.payload();
        let id0 = active_request_0.header().client_id();
        let p1 = *active_request_1.payload();
        let id1 = active_request_1.header().client_id();

        assert_that!(test.clients[p0].id(), eq id0);
        assert_that!(test.clients[p1].id(), eq id1);
    }

    #[test]
    fn server_port_ids_are_set_correctly<Sut: Service>() {
        let test_args = Args {
            number_of_servers: 2,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let pending_response = test.clients[0].send_copy(0).unwrap();
        let active_request_0 = test.servers[0].receive().unwrap().unwrap();
        let active_request_1 = test.servers[1].receive().unwrap().unwrap();

        assert_that!(active_request_0.send_copy(0), is_ok);
        assert_that!(active_request_1.send_copy(1), is_ok);

        let response0 = pending_response.receive().unwrap().unwrap();
        let response1 = pending_response.receive().unwrap().unwrap();

        let p0 = *response0.payload();
        let id0 = response0.header().server_id();
        let p1 = *response1.payload();
        let id1 = response1.header().server_id();

        assert_that!(test.servers[p0].id(), eq id0);
        assert_that!(test.servers[p1].id(), eq id1);
    }

    #[test]
    fn sent_responses_from_disconnected_servers_can_be_received<Sut: Service>() {
        let test_args = Args {
            response_buffer_size: 8,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);

        let pending_response = test.clients[0].send_copy(0).unwrap();
        let active_request = test.servers[0].receive().unwrap().unwrap();

        for n in 0..test_args.response_buffer_size {
            assert_that!(active_request.send_copy(n), is_ok);
        }

        // disconnect all servers
        test.servers.clear();

        for n in 0..test_args.response_buffer_size {
            assert_that!(*pending_response.receive().unwrap().unwrap(), eq n);
        }
    }

    #[test]
    fn sent_requests_from_disconnected_clients_can_be_received<Sut: Service>() {
        let test_args = Args {
            number_of_active_requests: 8,
            enable_fire_and_forget: true,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);
        assert_that!(test.servers[0].receive(), is_ok);

        for n in 0..test_args.number_of_active_requests {
            test.clients[0].send_copy(n).unwrap();
        }

        // disconnect all clients
        test.clients.clear();

        for n in 0..test_args.number_of_active_requests {
            let active_request = test.servers[0].receive().unwrap().unwrap();
            assert_that!(*active_request.payload(), eq n);
            assert_that!(active_request.is_connected(), eq false);
        }
    }

    #[test]
    fn sent_requests_from_disconnected_clients_are_not_received_without_fire_and_forget<
        Sut: Service,
    >() {
        let test_args = Args {
            number_of_active_requests: 8,
            enable_fire_and_forget: false,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);
        assert_that!(test.servers[0].receive(), is_ok);

        for n in 0..test_args.number_of_active_requests {
            test.clients[0].send_copy(n).unwrap();
        }

        // disconnect all clients
        test.clients.clear();

        let active_request = test.servers[0].receive().unwrap();
        assert_that!(active_request, is_none);
    }

    #[test]
    fn sent_requests_from_disconnected_clients_are_received_first<Sut: Service>() {
        let test_args = Args {
            number_of_active_requests: 8,
            number_of_clients: 2,
            enable_fire_and_forget: true,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);
        assert_that!(test.servers[0].receive(), is_ok);

        let mut pending_responses = vec![];
        for n in 0..test_args.number_of_active_requests {
            pending_responses.push(test.clients[0].send_copy(n + 100).unwrap());
            test.clients[1].send_copy(n).unwrap();
        }

        // disconnect last client in vec
        test.clients.pop();

        for n in 0..test_args.number_of_active_requests {
            let active_request = test.servers[0].receive().unwrap().unwrap();
            assert_that!(*active_request.payload(), eq n);
            assert_that!(active_request.is_connected(), eq false);
        }

        for n in 0..test_args.number_of_active_requests {
            let active_request = test.servers[0].receive().unwrap().unwrap();
            assert_that!(*active_request.payload(), eq n + 100);
            assert_that!(active_request.is_connected(), eq true);
        }
    }

    #[test]
    fn sent_responses_from_disconnected_servers_are_received_first<Sut: Service>() {
        let test_args = Args {
            number_of_servers: 2,
            response_buffer_size: 8,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);

        let pending_response = test.clients[0].send_copy(0).unwrap();
        let active_request_0 = test.servers[0].receive().unwrap().unwrap();
        let active_request_1 = test.servers[1].receive().unwrap().unwrap();

        for n in 0..test_args.response_buffer_size {
            assert_that!(active_request_0.send_copy(n), is_ok);
            assert_that!(active_request_1.send_copy(n + 100), is_ok);
        }

        // disconnect servers[1] and the request-response connection
        test.servers.pop();
        drop(active_request_1);

        for n in 0..test_args.response_buffer_size {
            assert_that!(*pending_response.receive().unwrap().unwrap(), eq n + 100);
            assert_that!(pending_response.is_connected(), eq true);
        }

        for n in 0..test_args.response_buffer_size {
            assert_that!(*pending_response.receive().unwrap().unwrap(), eq n );
            assert_that!(pending_response.is_connected(), eq true);
        }

        drop(active_request_0);
        assert_that!(pending_response.is_connected(), eq false);
    }

    #[test]
    fn sent_requests_from_out_of_scope_pending_responses_are_discarded_when_fire_and_forget_is_disabled<
        Sut: Service,
    >() {
        let test_args = Args {
            number_of_active_requests: 3,
            enable_fire_and_forget: false,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let pending_response_0 = test.clients[0].send_copy(5).unwrap();
        let pending_response_1 = test.clients[0].send_copy(7).unwrap();
        let pending_response_2 = test.clients[0].send_copy(11).unwrap();

        drop(pending_response_1);

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_0.payload());

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_2.payload());
    }

    #[test]
    fn sent_requests_from_out_of_scope_pending_responses_are_received_when_fire_and_forget_is_allowed<
        Sut: Service,
    >() {
        let test_args = Args {
            number_of_active_requests: 3,
            enable_fire_and_forget: true,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let pending_response_0 = test.clients[0].send_copy(5).unwrap();
        let pending_response_1 = test.clients[0].send_copy(7).unwrap();
        let pending_response_2 = test.clients[0].send_copy(11).unwrap();

        drop(pending_response_1);

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_0.payload());

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq 7);

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_2.payload());
    }

    #[test]
    fn sent_requests_from_out_of_scope_clients_are_not_discarded<Sut: Service>() {
        let test_args = Args {
            number_of_clients: 3,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);

        let pending_response_0 = test.clients[0].send_copy(5).unwrap();
        let pending_response_1 = test.clients[1].send_copy(7).unwrap();
        let pending_response_2 = test.clients[2].send_copy(11).unwrap();

        test.clients.remove(1);

        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_0.payload());
        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_1.payload());
        let active_request = test.servers[0].receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq * pending_response_2.payload());
    }

    #[test]
    fn responses_can_be_received_when_client_no_longer_exists<Sut: Service>() {
        let test_args = Args {
            response_buffer_size: 5,
            ..Default::default()
        };

        let mut test = TestFixture::<Sut>::new(test_args);

        let pending_response = test.clients[0].send_copy(5).unwrap();
        test.clients.clear();

        let active_request = test.servers[0].receive().unwrap().unwrap();
        for n in 0..test_args.response_buffer_size {
            assert_that!(active_request.send_copy(4 * n * n + 3), is_ok);
        }

        for n in 0..test_args.response_buffer_size {
            assert_that!(*pending_response.receive().unwrap().unwrap(), eq 4 * n * n + 3)
        }
    }

    #[test]
    fn safe_overflow_for_requests_works<Sut: Service>() {
        let test_args = Args {
            number_of_active_requests: 5,
            request_overflow: true,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        // send dummy data first so that the buffer is full and can overflow
        for _ in 0..test_args.number_of_active_requests {
            assert_that!(test.clients[0].send_copy(0), is_ok);
        }

        // let the buffer overflow
        let mut pending_responses = vec![];
        for n in 0..test_args.number_of_active_requests {
            pending_responses.push(test.clients[0].send_copy(n * n * n + 3).unwrap());
        }

        for n in 0..test_args.number_of_active_requests {
            assert_that!(*test.servers[0].receive().unwrap().unwrap(), eq n * n * n + 3);
        }
    }

    #[test]
    fn safe_overflow_for_responses_works<Sut: Service>() {
        let test_args = Args {
            response_buffer_size: 7,
            response_overflow: true,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);
        let pending_response = test.clients[0].send_copy(0).unwrap();
        let active_request = test.servers[0].receive().unwrap().unwrap();

        // send dummy data first so that the buffer is full and can overflow
        for _ in 0..test_args.response_buffer_size {
            assert_that!(active_request.send_copy(0), is_ok);
        }

        // let the buffer overflow
        for n in 0..test_args.response_buffer_size {
            assert_that!(active_request.send_copy(4 * n + 3 * n * n), is_ok);
        }

        for n in 0..test_args.response_buffer_size {
            assert_that!(*pending_response.receive().unwrap().unwrap(), eq 4 * n + 3 * n * n);
        }
    }

    #[test]
    fn communication_with_max_clients_and_servers_works<Sut: Service>() {
        const MAX_CLIENTS: usize = 4;
        const MAX_SERVERS: usize = 4;
        const MAX_ACTIVE_REQUESTS: usize = 2;

        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();

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
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let server = sut.server_builder().create().unwrap();
        let client = sut.client_builder().create().unwrap();

        drop(sut);

        let _pending_response = client.send_copy(8182982);
        assert_that!(*server.receive().unwrap().unwrap(), eq 8182982);
    }

    #[test]
    fn dropping_service_keeps_established_communication_for_active_requests<Sut: Service>() {
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let server = sut.server_builder().create().unwrap();
        let client = sut.client_builder().create().unwrap();

        drop(sut);

        let pending_response = client.send_copy(8182982).unwrap();
        let active_request = server.receive().unwrap().unwrap();

        drop(server);
        drop(client);

        assert_that!(active_request.send_copy(78223), is_ok);
        assert_that!(*pending_response.receive().unwrap().unwrap(), eq 78223);
    }

    #[test]
    fn requests_are_correctly_aligned_on_all_ends<Sut: Service>() {
        let test_args = Args {
            number_of_clients: 2,
            number_of_active_requests: 8,
            request_alignment: Alignment::new(512).unwrap(),
            ..Default::default()
        };
        let mut test = TestFixture::<Sut>::new(test_args);
        test.clients.pop();

        let service_2 = test
            .node
            .service_builder(test.service.name())
            .request_response::<usize, usize>()
            .request_user_header::<usize>()
            .response_user_header::<usize>()
            .open()
            .unwrap();

        let client_2 = service_2.client_builder().create().unwrap();

        for _ in 0..test_args.number_of_active_requests {
            let request = client_2.loan().unwrap();
            assert_that!(request.payload() as *const _, aligned_to test_args.request_alignment.value());
            assert_that!(request.send(), is_ok);
        }

        while let Some(request) = test.servers[0].receive().unwrap() {
            assert_that!(request.payload() as *const _, aligned_to test_args.request_alignment.value());
        }
    }

    #[test]
    fn responses_are_correctly_aligned_on_all_ends<Sut: Service>() {
        let test_args = Args {
            number_of_clients: 2,
            response_buffer_size: 21,
            response_alignment: Alignment::new(512).unwrap(),
            ..Default::default()
        };
        let mut test = TestFixture::<Sut>::new(test_args);
        test.clients.pop();

        let service_2 = test
            .node
            .service_builder(test.service.name())
            .request_response::<usize, usize>()
            .request_user_header::<usize>()
            .response_user_header::<usize>()
            .open()
            .unwrap();

        let client_2 = service_2.client_builder().create().unwrap();

        let request = client_2.send_copy(0).unwrap();
        let active_request = test.servers[0].receive().unwrap().unwrap();

        for _ in 0..test_args.response_buffer_size {
            let response = active_request.loan().unwrap();
            assert_that!((response.payload() as *const _), aligned_to test_args.response_alignment.value());
            assert_that!(response.send(), is_ok);
        }

        while let Some(response) = request.receive().unwrap() {
            assert_that!((response.payload() as *const _), aligned_to test_args.response_alignment.value());
        }
    }

    #[test]
    fn request_response_comm_with_mixed_types_works<Sut: Service>() {
        const REQUEST_PAYLOAD: u128 = 9891238912831298319823;
        const RESPONSE_PAYLOAD: u16 = 17821;
        const REQUEST_HEADER: u32 = 89213998;
        const RESPONSE_HEADER: u64 = 467440737095516161;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<u128, u16>()
            .request_user_header::<u32>()
            .response_user_header::<u64>()
            .create()
            .unwrap();

        let server = service.server_builder().create().unwrap();
        let client = service.client_builder().create().unwrap();

        let mut request = client.loan().unwrap();
        *request.payload_mut() = REQUEST_PAYLOAD;
        *request.user_header_mut() = REQUEST_HEADER;
        let pending_response = request.send().unwrap();

        let active_request = server.receive().unwrap().unwrap();
        assert_that!(*active_request.payload(), eq REQUEST_PAYLOAD);
        assert_that!(*active_request.user_header(), eq REQUEST_HEADER);

        let mut response = active_request.loan().unwrap();
        *response.payload_mut() = RESPONSE_PAYLOAD;
        *response.user_header_mut() = RESPONSE_HEADER;
        assert_that!(response.send(), is_ok);

        let response = pending_response.receive().unwrap().unwrap();
        assert_that!(*response.payload(), eq RESPONSE_PAYLOAD);
        assert_that!(*response.user_header(), eq RESPONSE_HEADER);
    }

    #[test]
    fn server_can_receive_max_amount_of_requests_from_max_clients<Sut: Service>() {
        let test_args = Args {
            number_of_active_requests: 5,
            number_of_clients: 6,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let mut pending_responses = vec![];
        let mut requests = vec![];
        let mut counter = 0;
        for client in &test.clients {
            for _ in 0..test_args.number_of_active_requests {
                pending_responses.push(client.send_copy(counter).unwrap());
                requests.push(counter);
                counter += 1;
            }
        }

        let mut active_requests = vec![];
        while let Some(request) = test.servers[0].receive().unwrap() {
            active_requests.push(request);
        }

        assert_that!(active_requests, len requests.len());
        for active_request in active_requests {
            assert_that!(requests, contains * active_request);
            requests.retain(|v| *v != *active_request);
        }
    }

    #[test]
    fn client_can_receive_max_amount_of_responses_from_max_servers<Sut: Service>() {
        let test_args = Args {
            number_of_active_requests: 3,
            response_buffer_size: 4,
            number_of_clients: 5,
            number_of_servers: 6,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        let mut pending_responses = vec![];
        let mut requests = vec![];
        let mut counter = 0;
        for client in &test.clients {
            for _ in 0..test_args.number_of_active_requests {
                pending_responses.push(client.send_copy(counter).unwrap());
                requests.push(counter);
                counter += 1;
            }
        }

        let mut active_requests = vec![];
        for server in &test.servers {
            while let Some(request) = server.receive().unwrap() {
                active_requests.push(request);
            }
        }

        assert_that!(active_requests, len requests.len() * test_args.number_of_servers);
        for active_request in &active_requests {
            assert_that!(requests, contains * *active_request);
        }

        let mut responses = vec![];
        for active_request in active_requests {
            for _ in 0..test_args.response_buffer_size {
                assert_that!(active_request.send_copy(counter), is_ok);
                responses.push(counter);
                counter += 1;
            }
        }

        let mut received_responses = vec![];
        for pending_response in &pending_responses {
            while let Some(response) = pending_response.receive().unwrap() {
                received_responses.push(*response);
            }
        }

        assert_that!(received_responses, len test_args.response_buffer_size * test_args.number_of_servers * test_args.number_of_active_requests * test_args.number_of_clients);
        for received_response in received_responses {
            assert_that!(responses, contains received_response);
        }
    }

    #[test]
    fn pending_response_receives_only_responses_from_sent_request<Sut: Service>() {
        const CONNECTION_ITERATIONS: usize = 6;
        const SEND_RECEIVE_ITERATIONS: usize = 8;
        let test_args = Args {
            number_of_active_requests: 7,
            ..Default::default()
        };

        let test = TestFixture::<Sut>::new(test_args);

        for _ in 0..CONNECTION_ITERATIONS {
            let mut pending_responses = vec![];
            for n in 0..test_args.number_of_active_requests {
                pending_responses.push(test.clients[0].send_copy(n * 51).unwrap());
            }

            let mut active_requests = vec![];
            while let Some(request) = test.servers[0].receive().unwrap() {
                active_requests.push(request);
            }

            for n in 0..SEND_RECEIVE_ITERATIONS {
                for active_request in &active_requests {
                    active_request
                        .send_copy(active_request.payload() + n)
                        .unwrap();
                }

                for pending_response in &pending_responses {
                    let response = pending_response.receive().unwrap().unwrap();
                    assert_that!(*response.payload(), eq pending_response.payload() + n);
                    assert_that!(pending_response.receive().unwrap(), is_none);
                }
            }
        }
    }

    #[test]
    fn sending_requests_with_custom_payload_works<Sut: Service>() {
        const NUMBER_OF_ELEMENTS: usize = 1;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_details = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        testing::type_detail_set_size(&mut type_details, 1024);
        testing::type_detail_set_alignment(&mut type_details, 1024);

        let service_1 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_request_payload_type_details(&type_details)
                .create()
                .unwrap()
        };
        let service_2 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_request_payload_type_details(&type_details)
                .open()
                .unwrap()
        };

        let server = service_1.server_builder().create().unwrap();
        let client = service_2.client_builder().create().unwrap();

        let mut request = unsafe { client.loan_custom_payload(NUMBER_OF_ELEMENTS).unwrap() };
        assert_that!(request.payload(), len type_details.size());
        assert_that!((request.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(request.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);
        let payload_ptr = request.payload_mut().as_mut_ptr() as *mut u8;
        for n in 0..type_details.size() {
            unsafe { payload_ptr.add(n).write((n % 255) as u8) };
        }

        let _pending_response = unsafe { request.assume_init().send().unwrap() };

        let active_request = unsafe { server.receive_custom_payload().unwrap().unwrap() };
        assert_that!(active_request.payload(), len type_details.size());
        assert_that!((active_request.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(active_request.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);
        let payload_ptr = active_request.payload().as_ptr() as *const u8;
        for n in 0..type_details.size() {
            assert_that!(unsafe { *payload_ptr.add(n) }, eq(n % 255) as u8);
        }
    }

    #[test]
    fn sending_response_with_custom_payload_works<Sut: Service>() {
        const NUMBER_OF_ELEMENTS: usize = 1;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_details = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        testing::type_detail_set_size(&mut type_details, 512);
        testing::type_detail_set_alignment(&mut type_details, 256);

        let service_1 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_response_payload_type_details(&type_details)
                .create()
                .unwrap()
        };
        let service_2 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_response_payload_type_details(&type_details)
                .open()
                .unwrap()
        };

        let server = service_1.server_builder().create().unwrap();
        let client = service_2.client_builder().create().unwrap();

        let request = unsafe { client.loan_custom_payload(NUMBER_OF_ELEMENTS).unwrap() };
        let pending_response = unsafe { request.assume_init().send().unwrap() };
        let active_request = unsafe { server.receive_custom_payload().unwrap().unwrap() };

        let mut response = unsafe {
            active_request
                .loan_custom_payload(NUMBER_OF_ELEMENTS)
                .unwrap()
        };
        assert_that!(response.payload(), len type_details.size());
        assert_that!((response.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(response.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);
        let payload_ptr = response.payload_mut().as_mut_ptr() as *mut u8;
        for n in 0..type_details.size() {
            unsafe { payload_ptr.add(n).write((n % 89) as u8) };
        }

        unsafe { response.assume_init().send().unwrap() };

        let response = unsafe { pending_response.receive_custom_payload().unwrap().unwrap() };
        assert_that!(response.payload(), len type_details.size());
        assert_that!((response.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(response.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);
        let payload_ptr = response.payload().as_ptr() as *const u8;
        for n in 0..type_details.size() {
            assert_that!(unsafe { *payload_ptr.add(n) }, eq(n % 89) as u8);
        }
    }

    #[test]
    fn sending_requests_with_custom_header_works<Sut: Service>() {
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_details = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        testing::type_detail_set_size(&mut type_details, 2048);
        testing::type_detail_set_alignment(&mut type_details, 8);

        let service_1 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_request_header_type_details(&type_details)
                .create()
                .unwrap()
        };
        let service_2 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_request_header_type_details(&type_details)
                .open()
                .unwrap()
        };

        let server = service_1.server_builder().create().unwrap();
        let client = service_2.client_builder().create().unwrap();

        let mut request = unsafe { client.loan_custom_payload(1).unwrap() };
        let header_ptr = (request.user_header_mut() as *mut CustomHeaderMarker) as *mut u8;
        for n in 0..type_details.size() {
            unsafe { header_ptr.add(n).write((n % 231) as u8) };
        }
        let _pending_response = unsafe { request.assume_init().send().unwrap() };

        let active_request = unsafe { server.receive_custom_payload().unwrap().unwrap() };
        let header_ptr = (active_request.user_header() as *const CustomHeaderMarker) as *const u8;
        for n in 0..type_details.size() {
            assert_that!(unsafe { *header_ptr.add(n) }, eq(n % 231) as u8);
        }
    }

    #[test]
    fn sending_response_with_custom_header_works<Sut: Service>() {
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_details = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        testing::type_detail_set_size(&mut type_details, 4096);
        testing::type_detail_set_alignment(&mut type_details, 32);

        let service_1 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_response_header_type_details(&type_details)
                .create()
                .unwrap()
        };
        let service_2 = unsafe {
            node.service_builder(&service_name)
                .request_response::<[CustomPayloadMarker], [CustomPayloadMarker]>()
                .request_user_header::<CustomHeaderMarker>()
                .response_user_header::<CustomHeaderMarker>()
                .__internal_set_response_header_type_details(&type_details)
                .open()
                .unwrap()
        };

        let server = service_1.server_builder().create().unwrap();
        let client = service_2.client_builder().create().unwrap();

        let request = unsafe { client.loan_custom_payload(1).unwrap() };
        let pending_response = unsafe { request.assume_init().send().unwrap() };
        let active_request = unsafe { server.receive_custom_payload().unwrap().unwrap() };

        let mut response = unsafe { active_request.loan_custom_payload(1).unwrap() };
        let header_ptr = (response.user_header_mut() as *mut CustomHeaderMarker) as *mut u8;
        for n in 0..type_details.size() {
            unsafe { header_ptr.add(n).write((n % 229) as u8) };
        }
        unsafe { response.assume_init().send().unwrap() };

        let response = unsafe { pending_response.receive_custom_payload().unwrap().unwrap() };
        let header_ptr = (response.user_header() as *const CustomHeaderMarker) as *const u8;
        for n in 0..type_details.size() {
            assert_that!(unsafe { *header_ptr.add(n) }, eq(n % 229) as u8);
        }
    }

    #[test]
    fn send_increasing_requests_with_static_allocation_strategy_fails<Sut: Service>() {
        const SLICE_SIZE: usize = 1024;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<[u8], u64>()
            .create()
            .unwrap();

        let client = service
            .client_builder()
            .initial_max_slice_len(SLICE_SIZE)
            .allocation_strategy(AllocationStrategy::Static)
            .create()
            .unwrap();

        let request = client.loan_slice(SLICE_SIZE - 1);
        assert_that!(request, is_ok);

        let request = client.loan_slice(SLICE_SIZE);
        assert_that!(request, is_ok);

        let request = client.loan_slice(SLICE_SIZE + 1);
        assert_that!(request.err(), eq Some(LoanError::ExceedsMaxLoanSize));
    }

    #[test]
    fn send_increasing_responses_with_static_allocation_strategy_fails<Sut: Service>() {
        const SLICE_SIZE: usize = 1024;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<u64, [u8]>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let server = service
            .server_builder()
            .initial_max_slice_len(SLICE_SIZE)
            .allocation_strategy(AllocationStrategy::Static)
            .create()
            .unwrap();
        let _pending_response = client.send_copy(0).unwrap();
        let active_request = server.receive().unwrap().unwrap();

        let response = active_request.loan_slice(SLICE_SIZE - 1);
        assert_that!(response, is_ok);

        let response = active_request.loan_slice(SLICE_SIZE);
        assert_that!(response, is_ok);

        let response = active_request.loan_slice(SLICE_SIZE + 1);
        assert_that!(response.err(), eq Some(LoanError::ExceedsMaxLoanSize));
    }

    fn send_and_receive_increasing_requests_works<Sut: Service>(
        allocation_strategy: AllocationStrategy,
    ) {
        const ITERATIONS: usize = 128;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<[u8], u64>()
            .create()
            .unwrap();

        let client = service
            .client_builder()
            .initial_max_slice_len(1)
            .allocation_strategy(allocation_strategy)
            .create()
            .unwrap();
        let server = service.server_builder().create().unwrap();

        for n in 0..ITERATIONS {
            let request_size = (n + 1) * 32;
            let mut request = client.loan_slice(request_size).unwrap();
            for byte in request.payload_mut() {
                *byte = n as u8;
            }

            let _pending_response = request.send().unwrap();

            let active_request = server.receive().unwrap().unwrap();
            assert_that!(active_request.payload(), len request_size);
            for byte in active_request.payload() {
                assert_that!(*byte, eq n as u8);
            }
        }
    }

    #[test]
    fn send_and_receive_increasing_requests_with_best_fit_allocation_works<Sut: Service>() {
        send_and_receive_increasing_requests_works::<Sut>(AllocationStrategy::BestFit);
    }

    #[test]
    fn send_and_receive_increasing_requests_with_power_of_two_allocation_works<Sut: Service>() {
        send_and_receive_increasing_requests_works::<Sut>(AllocationStrategy::PowerOfTwo);
    }

    fn send_and_receive_increasing_responses_works<Sut: Service>(
        allocation_strategy: AllocationStrategy,
    ) {
        const ITERATIONS: usize = 128;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<u64, [u8]>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let server = service
            .server_builder()
            .initial_max_slice_len(1)
            .allocation_strategy(allocation_strategy)
            .create()
            .unwrap();
        let pending_response = client.send_copy(0).unwrap();
        let active_request = server.receive().unwrap().unwrap();

        for n in 0..ITERATIONS {
            let response_size = (n + 1) * 32;
            let mut response = active_request.loan_slice(response_size).unwrap();
            for byte in response.payload_mut() {
                *byte = n as u8 + 5;
            }
            response.send().unwrap();

            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(response.payload(), len response_size);
            for byte in response.payload() {
                assert_that!(*byte, eq n as u8 + 5);
            }
        }
    }

    #[test]
    fn send_and_receive_increasing_responses_with_best_fit_allocation_strategy_works<
        Sut: Service,
    >() {
        send_and_receive_increasing_responses_works::<Sut>(AllocationStrategy::BestFit);
    }

    #[test]
    fn send_and_receive_increasing_responses_with_power_of_two_allocation_strategy_works<
        Sut: Service,
    >() {
        send_and_receive_increasing_responses_works::<Sut>(AllocationStrategy::PowerOfTwo);
    }

    #[test]
    fn listing_all_clients_works<S: Service>() {
        const NUMBER_OF_CLIENTS: usize = 17;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(NUMBER_OF_CLIENTS)
            .create()
            .unwrap();

        let mut clients = vec![];

        for _ in 0..NUMBER_OF_CLIENTS {
            clients.push(sut.client_builder().create().unwrap());
        }

        let mut client_details = vec![];
        sut.dynamic_config().list_clients(|details| {
            client_details.push(details.client_id);
            CallbackProgression::Continue
        });

        assert_that!(client_details, len NUMBER_OF_CLIENTS);
        for client in clients {
            assert_that!(client_details, contains client.id());
        }
    }

    #[test]
    fn listing_all_clients_stops_on_request<S: Service>() {
        const NUMBER_OF_CLIENTS: usize = 13;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(NUMBER_OF_CLIENTS)
            .create()
            .unwrap();

        let mut clients = vec![];

        for _ in 0..NUMBER_OF_CLIENTS {
            clients.push(sut.client_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_clients(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn listing_all_servers_works<S: Service>() {
        const NUMBER_OF_SERVERS: usize = 17;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_servers(NUMBER_OF_SERVERS)
            .create()
            .unwrap();

        let mut servers = vec![];

        for _ in 0..NUMBER_OF_SERVERS {
            servers.push(sut.server_builder().create().unwrap());
        }

        let mut server_details = vec![];
        sut.dynamic_config().list_servers(|details| {
            server_details.push(details.server_id);
            CallbackProgression::Continue
        });

        assert_that!(server_details, len NUMBER_OF_SERVERS);
        for server in servers {
            assert_that!(server_details, contains server.id());
        }
    }

    #[test]
    fn listing_all_servers_stops_on_request<S: Service>() {
        const NUMBER_OF_SERVERS: usize = 13;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_servers(NUMBER_OF_SERVERS)
            .create()
            .unwrap();

        let mut servers = vec![];

        for _ in 0..NUMBER_OF_SERVERS {
            servers.push(sut.server_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_servers(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn receive_does_not_return_error_when_server_goes_out_of_scope_after_reallocation<
        S: Service,
    >() {
        const SLICE_MAX_LEN: usize = 1;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<u8, [u8]>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();

        let server = service
            .server_builder()
            .initial_max_slice_len(SLICE_MAX_LEN)
            .allocation_strategy(AllocationStrategy::BestFit)
            .create()
            .unwrap();

        let pending_response = client.send_copy(0).unwrap();

        // send and receive once so that the client maps the data segment
        let active_request = server.receive().unwrap().unwrap();
        active_request
            .loan_slice(SLICE_MAX_LEN)
            .unwrap()
            .send()
            .unwrap();

        let _received_response = pending_response.receive().unwrap().unwrap();

        // server has to reallocate the data segment
        active_request
            .loan_slice(SLICE_MAX_LEN + 4096)
            .unwrap()
            .send()
            .unwrap();
        // server goes out of scope and closes the reallocated data segment as it was not yet mapped by the client
        drop(server);
        drop(active_request);

        let response = pending_response.receive();
        assert_that!(response, is_ok);
    }

    #[test]
    fn receive_does_not_return_error_when_client_goes_out_of_scope_after_reallocation<
        S: Service,
    >() {
        const SLICE_MAX_LEN: usize = 1;
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<[u8], u8>()
            .enable_fire_and_forget_requests(true)
            .create()
            .unwrap();

        let server = service.server_builder().create().unwrap();
        let mut active_request = server.receive();
        assert_that!(active_request, is_ok);

        let client = service
            .client_builder()
            .initial_max_slice_len(SLICE_MAX_LEN)
            .allocation_strategy(AllocationStrategy::BestFit)
            .create()
            .unwrap();

        // send and receive once so that the client maps the data segment
        client.loan_slice(SLICE_MAX_LEN).unwrap().send().unwrap();
        active_request = server.receive();
        assert_that!(active_request, is_ok);

        // client has to reallocate the data segment
        client
            .loan_slice(SLICE_MAX_LEN + 4096)
            .unwrap()
            .send()
            .unwrap();

        // client goes out of scope and closes the reallocated data segment as it was not yet mapped by the server
        drop(client);

        active_request = server.receive();
        assert_that!(active_request, is_ok);
    }

    #[test]
    fn client_can_set_disconnect_hint<S: Service>() {
        let service_name = testing::generate_service_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let service = node
            .service_builder(&service_name)
            .request_response::<usize, usize>()
            .create()
            .unwrap();

        let server = service.server_builder().create().unwrap();
        let client = service.client_builder().create().unwrap();

        let pending_response = client.send_copy(123).unwrap();
        let active_request = server.receive().unwrap().unwrap();

        assert_that!(pending_response.is_connected(), eq true);
        assert_that!(active_request.is_connected(), eq true);
        assert_that!(active_request.has_disconnect_hint(), eq false);

        pending_response.set_disconnect_hint();

        assert_that!(pending_response.is_connected(), eq true);
        assert_that!(active_request.is_connected(), eq true);
        assert_that!(active_request.has_disconnect_hint(), eq true);

        drop(pending_response);

        assert_that!(active_request.is_connected(), eq false);
        assert_that!(active_request.has_disconnect_hint(), eq false);
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
