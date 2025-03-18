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

// TODO:
//  service
//    -fn concurrent_communication_with_subscriber_reconnects_does_not_deadlock

#[generic_tests::define]
mod service_request_response {
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

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
