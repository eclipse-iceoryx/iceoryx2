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
    use core::sync::atomic::Ordering;
    use core::time::Duration;
    use std::sync::Barrier;

    use iceoryx2::port::ReceiveError;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

    const TIMEOUT: Duration = Duration::from_millis(50);

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
            .enable_fire_and_forget_requests(false)
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
        let _pending_response = client.send_copy(1234).unwrap();
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
        let _pending_response = client.send_copy(5678);
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
        let mut pending_responses = vec![];

        for client in clients {
            for n in 0..max_active_requests {
                pending_responses.push(client.send_copy(n as u64 * 5 + 7).unwrap());
                let active_request = sut.receive().unwrap().unwrap();
                assert_that!(*active_request, eq n as u64 * 5 + 7);
                active_requests.push(active_request);
            }

            pending_responses.pop();
            pending_responses.push(client.send_copy(99).unwrap());
            let active_request = sut.receive();
            assert_that!(active_request.err(), eq Some(ReceiveError::ExceedsMaxBorrows));
        }
    }

    #[test]
    fn max_loaned_responses_per_requests_is_adjusted_to_sane_values<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();

        let sut = service
            .server_builder()
            .max_loaned_responses_per_request(0)
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let _pending_response = client.send_copy(0).unwrap();

        let active_request = sut.receive().unwrap().unwrap();
        // max loaned responses per request needs to be adjusted to 1 and therefore this
        // must work
        assert_that!(active_request.loan(), is_ok);
    }

    #[test]
    fn can_loan_at_most_max_responses_per_requests<Sut: Service>() {
        const MAX_LOANS: usize = 5;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let sut = service
            .server_builder()
            .max_loaned_responses_per_request(MAX_LOANS)
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let _pending_response = client.send_copy(0).unwrap();

        let active_request = sut.receive().unwrap().unwrap();

        let mut loans = vec![];
        for _ in 0..MAX_LOANS {
            loans.push(active_request.loan().unwrap());
        }

        assert_that!(active_request.loan().err(), eq Some(iceoryx2::port::LoanError::ExceedsMaxLoans));
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

    #[test]
    fn unable_to_deliver_strategy_discard_discards_responses_when_client_buffer_is_full<
        Sut: Service,
    >() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_response_buffer_size(1)
            .enable_safe_overflow_for_responses(false)
            .create()
            .unwrap();

        let sut = service
            .server_builder()
            .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let pending_response = client.send_copy(123).unwrap();

        let active_request = sut.receive().unwrap().unwrap();

        for n in 5..15 {
            assert_that!(active_request.send_copy(n), is_ok);
        }

        let response = pending_response.receive().unwrap().unwrap();
        assert_that!(*response, eq 5);

        let response = pending_response.receive().unwrap();
        assert_that!(response, is_none);
    }

    #[test]
    fn unable_to_deliver_strategy_block_blocks_responses_when_client_buffer_is_full<
        Sut: Service,
    >() {
        let _watchdog = Watchdog::new();
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_response_buffer_size(1)
            .enable_safe_overflow_for_responses(false)
            .create()
            .unwrap();
        let client = service.client_builder().create().unwrap();
        let barrier = Barrier::new(2);
        let send_barrier = Barrier::new(2);

        let has_sent_response = IoxAtomicBool::new(false);
        std::thread::scope(|s| {
            s.spawn(|| {
                let sut = service
                    .server_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
                    .create()
                    .unwrap();
                barrier.wait();
                send_barrier.wait();

                let active_request = sut.receive().unwrap().unwrap();
                assert_that!(active_request.send_copy(321), is_ok);

                assert_that!(active_request.send_copy(654), is_ok);
                has_sent_response.store(true, Ordering::Relaxed);
            });

            barrier.wait();
            let pending_response = client.send_copy(123).unwrap();
            send_barrier.wait();

            std::thread::sleep(TIMEOUT);

            assert_that!(has_sent_response.load(Ordering::Relaxed), eq false);
            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response, eq 321);
            assert_that!(|| has_sent_response.load(Ordering::Relaxed), block_until true);

            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response, eq 654);
        });
    }

    #[test]
    fn reclaims_all_responses_delivered_to_client_after_a_client_disconnect<Sut: Service>() {
        const MAX_ACTIVE_REQUESTS: usize = 4;
        const ITERATIONS: usize = 20;
        const MAX_CLIENTS: usize = 4;
        const RESPONSE_BUFFER_SIZE: usize = 7;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
            .max_clients(MAX_CLIENTS)
            .max_response_buffer_size(RESPONSE_BUFFER_SIZE)
            .create()
            .unwrap();

        let sut = service.server_builder().create().unwrap();

        for n in 0..MAX_CLIENTS {
            for _ in 0..ITERATIONS {
                let mut requests = vec![];
                let mut clients = vec![];
                for _ in 0..n {
                    let client = service.client_builder().create().unwrap();

                    for _ in 0..MAX_ACTIVE_REQUESTS {
                        requests.push(client.send_copy(0).unwrap());
                    }

                    clients.push(client);
                }

                while let Some(request) = sut.receive().unwrap() {
                    for _ in 0..RESPONSE_BUFFER_SIZE {
                        request.send_copy(0).unwrap();
                    }
                }

                // disconnect all clients by dropping them and the requests
                drop(requests);
                drop(clients);
            }
        }
    }

    #[test]
    fn updates_connections_after_reconnect<Sut: Service>() {
        const ITERATIONS: usize = 20;
        const MAX_CLIENTS: usize = 4;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(MAX_CLIENTS)
            .create()
            .unwrap();

        let sut = service.server_builder().create().unwrap();

        for _ in 0..ITERATIONS {
            let mut requests = vec![];
            let mut clients = vec![];
            for _ in 0..MAX_CLIENTS {
                let client = service.client_builder().create().unwrap();
                requests.push(client.send_copy(0).unwrap());
                clients.push(client);

                assert_that!(sut.receive().unwrap(), is_some);
            }
        }
    }

    fn completion_channel_capacity_is_never_exceeded_impl<Sut: Service>(
        max_response_buffer_size: usize,
        max_borrowed_responses: usize,
    ) {
        const ITERATIONS: usize = 5;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(1)
            .max_servers(1)
            .max_response_buffer_size(max_response_buffer_size)
            .max_active_requests_per_client(1)
            .max_borrowed_responses_per_pending_response(max_borrowed_responses)
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let sut = service.server_builder().create().unwrap();

        let pending_response = client.send_copy(0).unwrap();
        let active_request = sut.receive().unwrap().unwrap();

        for _ in 0..ITERATIONS {
            let mut borrowed_responses = vec![];
            for _ in 0..max_borrowed_responses {
                assert_that!(active_request.send_copy(0), is_ok);
                borrowed_responses.push(pending_response.receive().unwrap().unwrap());
            }

            for _ in 0..max_response_buffer_size {
                assert_that!(active_request.send_copy(0), is_ok);
            }

            // return everything at once and verify that in the worst case scenario
            // the completion channel capacity is sufficient
            drop(borrowed_responses);
            for _ in 0..max_response_buffer_size {
                assert_that!(pending_response.receive().unwrap(), is_some);
            }
        }
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_buffer_size<Sut: Service>() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 100;
        const MAX_BORROWED_RESPONSES: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_response_borrow_size<
        Sut: Service,
    >() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 100;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_values<Sut: Service>() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 100;
        const MAX_BORROWED_RESPONSES: usize = 100;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_smallest_possible_values<Sut: Service>() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    fn server_never_goes_out_of_memory_impl<Sut: Service>(
        max_clients: usize,
        max_active_requests: usize,
        max_response_buffer_size: usize,
        max_borrowed_responses: usize,
        max_loaned_responses: usize,
    ) {
        const ITERATIONS: usize = 5;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(max_clients)
            .max_servers(1)
            .max_response_buffer_size(max_response_buffer_size)
            .max_active_requests_per_client(max_active_requests)
            .max_borrowed_responses_per_pending_response(max_borrowed_responses)
            .create()
            .unwrap();

        let mut clients = vec![];

        for _ in 0..max_clients {
            clients.push(service.client_builder().create().unwrap());
        }
        let sut = service
            .server_builder()
            .max_loaned_responses_per_request(max_loaned_responses)
            .create()
            .unwrap();

        for _ in 0..ITERATIONS {
            let mut pending_responses = vec![];
            for client in &clients {
                for _ in 0..max_active_requests {
                    pending_responses.push(client.send_copy(0).unwrap());
                }
            }

            let mut active_requests = vec![];
            for _ in 0..max_active_requests * max_clients {
                active_requests.push(sut.receive().unwrap().unwrap());
            }

            // borrow maximum amount of responses
            let mut responses = vec![];
            for _ in 0..max_borrowed_responses {
                for active_request in &active_requests {
                    assert_that!(active_request.send_copy(0), is_ok);
                }
                for pending_response in &pending_responses {
                    responses.push(pending_response.receive().unwrap().unwrap());
                }
            }

            // fill response buffer size
            for _ in 0..max_borrowed_responses {
                for active_request in &active_requests {
                    assert_that!(active_request.send_copy(0), is_ok);
                }
            }

            // loan maximum amount of responses per request
            let mut loaned_responses = vec![];
            for active_request in &active_requests {
                for _ in 0..max_loaned_responses {
                    loaned_responses.push(active_request.loan().unwrap());
                }
            }
        }
    }

    #[test]
    fn server_runs_never_out_of_memory_with_smallest_possible_values<Sut: Service>() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 1;
        const MAX_LOANED_RESPONSES: usize = 1;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn server_runs_never_out_of_memory_with_huge_values<Sut: Service>() {
        const MAX_CLIENTS: usize = 10;
        const MAX_ACTIVE_REQUESTS: usize = 10;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 10;
        const MAX_BORROWED_RESPONSES: usize = 10;
        const MAX_LOANED_RESPONSES: usize = 10;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn server_runs_never_out_of_memory_with_huge_max_clients<Sut: Service>() {
        const MAX_CLIENTS: usize = 10;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 1;
        const MAX_LOANED_RESPONSES: usize = 1;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn server_runs_never_out_of_memory_with_huge_max_active_requests<Sut: Service>() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 10;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 1;
        const MAX_LOANED_RESPONSES: usize = 1;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn server_runs_never_out_of_memory_with_huge_response_buffer_size<Sut: Service>() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 10;
        const MAX_BORROWED_RESPONSES: usize = 1;
        const MAX_LOANED_RESPONSES: usize = 1;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn server_runs_never_out_of_memory_with_huge_borrowed_responses<Sut: Service>() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 10;
        const MAX_LOANED_RESPONSES: usize = 1;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn server_runs_never_out_of_memory_with_huge_max_loaned_responses<Sut: Service>() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 1;
        const MAX_LOANED_RESPONSES: usize = 10;

        server_never_goes_out_of_memory_impl::<Sut>(
            MAX_CLIENTS,
            MAX_ACTIVE_REQUESTS,
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
            MAX_LOANED_RESPONSES,
        );
    }

    #[test]
    fn can_loan_per_request_at_most_number_of_responses<Sut: Service>() {
        const MAX_CLIENTS: usize = 4;
        const MAX_ACTIVE_REQUESTS: usize = 5;
        const MAX_LOANED_RESPONSES: usize = 6;
        const ITERATIONS: usize = 5;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(MAX_CLIENTS)
            .max_servers(1)
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
            .create()
            .unwrap();

        let sut = service
            .server_builder()
            .max_loaned_responses_per_request(MAX_LOANED_RESPONSES)
            .create()
            .unwrap();

        let mut clients = vec![];
        for _ in 0..MAX_CLIENTS {
            clients.push(service.client_builder().create().unwrap());
        }

        for _ in 0..ITERATIONS {
            let mut pending_responses = vec![];
            for client in &clients {
                for _ in 0..MAX_ACTIVE_REQUESTS {
                    pending_responses.push(client.send_copy(0).unwrap());
                }
            }

            let mut active_requests = vec![];
            for _ in 0..MAX_ACTIVE_REQUESTS * MAX_CLIENTS {
                active_requests.push(sut.receive().unwrap().unwrap());
            }

            let mut loans = vec![];
            for active_request in &active_requests {
                for _ in 0..MAX_LOANED_RESPONSES {
                    loans.push(active_request.loan().unwrap());
                }
            }

            assert_that!(loans, len MAX_LOANED_RESPONSES * MAX_ACTIVE_REQUESTS * MAX_CLIENTS);
        }
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
