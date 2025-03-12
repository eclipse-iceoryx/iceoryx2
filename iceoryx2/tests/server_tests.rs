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

    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;

    // TODO:
    //   - server
    //     - test that it can hold X active requests per client (for one and many clients)
    //       - holding more requests leads to failure
    //
    //   - service builder
    //     - ports of dropped service block new service creation
    //     - service can be opened when there is a port
    //     - ?server decrease buffer size? (increase with failure)
    //     - create max amount of ports
    //     - create max amount of nodes
    //
    //   - service
    //    -fn concurrent_communication_with_subscriber_reconnects_does_not_deadlock
    //    - service can be open when there is a server/client
    //    - dropping service keeps established comm
    //    - comm with max clients/server
    //     - disconnected server does not block new server
    //     - receive requests client created first
    //       - server created first

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

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
