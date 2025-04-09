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
    use iceoryx2::port::client::Client;
    use iceoryx2::port::server::Server;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2::{pending_response, prelude::*};
    use iceoryx2_bb_testing::assert_that;

    struct Args {
        number_of_clients: usize,
        number_of_servers: usize,
    }

    impl Args {
        fn new() -> Self {
            Self {
                number_of_clients: 1,
                number_of_servers: 1,
            }
        }
    }

    struct TestFixture<Sut: Service> {
        _node: Node<Sut>,
        service: PortFactory<Sut, u64, u64, u64, ()>,
        clients: Vec<Client<Sut, u64, u64, u64, ()>>,
        servers: Vec<Server<Sut, u64, u64, u64, ()>>,
    }

    impl<Sut: Service> TestFixture<Sut> {
        fn new(args: Args) -> Self {
            let config = generate_isolated_config();
            let service_name = generate_service_name();
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let service = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .request_user_header::<u64>()
                .max_servers(args.number_of_servers)
                .max_clients(args.number_of_clients)
                .create()
                .unwrap();

            let mut clients = vec![];
            for _ in 0..args.number_of_clients {
                clients.push(service.client_builder().create().unwrap());
            }

            let mut servers = vec![];
            for _ in 0..args.number_of_servers {
                servers.push(service.server_builder().create().unwrap());
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
    fn request_response_stream_works<Sut: Service>() {
        let test_args = Args {
            number_of_clients: 5,
            number_of_servers: 7,
        };
        let test = TestFixture::<Sut>::new(test_args);

        let mut pending_responses = vec![];
        for (n, client) in test.clients.iter().enumerate() {
            pending_responses.push(client.send_copy(n as _).unwrap());
        }

        let mut active_responses = vec![];
        for server in test.servers.iter() {
            active_responses.push(server.receive().unwrap().unwrap());
        }

        for (n, active_response) in active_responses.iter().enumerate() {
            let response_value = n as u64 * *active_response.payload();
            active_response.send_copy(response_value).unwrap();

            for pending_response in &pending_responses {
                let response = pending_response.receive().unwrap().unwrap();
                assert_that!(*response.payload(), eq response_value);
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
