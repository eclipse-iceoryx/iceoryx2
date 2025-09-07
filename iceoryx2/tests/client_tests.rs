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
mod client {
    use std::ops::Deref;
    use std::sync::atomic::Ordering;
    use std::sync::Barrier;
    use std::time::Duration;

    use iceoryx2::port::client::RequestSendError;
    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

    const TIMEOUT: Duration = Duration::from_millis(50);

    fn create_node<Sut: Service>() -> Node<Sut> {
        let config = generate_isolated_config();
        NodeBuilder::new().config(&config).create::<Sut>().unwrap()
    }

    #[test]
    fn disconnected_client_does_not_block_new_clients<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(1)
            .create()
            .unwrap();

        let sut = service.client_builder().create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut = service.client_builder().create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn send_request_via_disconnected_client_works<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let request = sut.loan().unwrap();
        drop(sut);

        assert_that!(request.send(), is_ok);
    }

    #[test]
    fn can_loan_at_most_max_supported_amount_of_requests<Sut: Service>() {
        const MAX_LOANED_REQUESTS: usize = 29;
        const ITERATIONS: usize = 3;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(MAX_LOANED_REQUESTS)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        for _ in 0..ITERATIONS {
            let mut requests = vec![];
            for _ in 0..MAX_LOANED_REQUESTS {
                let request = sut.loan_uninit();
                assert_that!(request, is_ok);
                requests.push(request);
            }
            let request = sut.loan_uninit();
            assert_that!(request.err(), eq Some(LoanError::ExceedsMaxLoans));
        }
    }

    #[test]
    fn can_loan_max_supported_amount_of_requests_when_holding_max_pending_responses<
        Sut: Service,
    >() {
        const MAX_LOANED_REQUESTS: usize = 29;
        const MAX_PENDING_RESPONSES: usize = 7;
        const ITERATIONS: usize = 3;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(MAX_PENDING_RESPONSES)
            .max_loaned_requests(MAX_LOANED_REQUESTS)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        for _ in 0..ITERATIONS {
            let mut pending_responses = vec![];
            for _ in 0..MAX_PENDING_RESPONSES {
                pending_responses.push(sut.send_copy(123).unwrap());
            }

            let mut requests = vec![];
            for _ in 0..MAX_LOANED_REQUESTS {
                let request = sut.loan_uninit();
                assert_that!(request, is_ok);
                requests.push(request);
            }
            let request = sut.loan_uninit();
            assert_that!(request.err(), eq Some(LoanError::ExceedsMaxLoans));
        }
    }

    #[test]
    fn unable_to_deliver_strategy_block_blocks_when_server_buffer_is_full<Sut: Service>() {
        let _watchdog = Watchdog::new();
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(false)
            .max_active_requests_per_client(1)
            .create()
            .unwrap();
        let server = service.server_builder().create().unwrap();
        let has_sent_request = IoxAtomicBool::new(false);
        let barrier = Barrier::new(2);

        std::thread::scope(|s| {
            s.spawn(|| {
                let sut = service
                    .client_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
                    .create()
                    .unwrap();

                assert_that!(sut.unable_to_deliver_strategy(), eq UnableToDeliverStrategy::Block);

                let request = sut.send_copy(123);
                assert_that!(request, is_ok);
                drop(request);
                barrier.wait();

                let request = sut.send_copy(123);
                has_sent_request.store(true, Ordering::Relaxed);
                assert_that!(request, is_ok);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            assert_that!(has_sent_request.load(Ordering::Relaxed), eq false);
            let data = server.receive();
            assert_that!(data, is_ok);
            assert_that!(|| has_sent_request.load(Ordering::Relaxed), block_until true);
        });
    }

    #[test]
    fn unable_to_deliver_strategy_discard_discards_sample<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(false)
            .max_active_requests_per_client(1)
            .create()
            .unwrap();
        let server = service.server_builder().create().unwrap();

        let sut = service
            .client_builder()
            .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
            .create()
            .unwrap();

        assert_that!(sut.unable_to_deliver_strategy(), eq UnableToDeliverStrategy::DiscardSample);

        let request = sut.send_copy(123);
        assert_that!(request, is_ok);
        let _request = sut.send_copy(456);

        let data = server.receive();
        assert_that!(data, is_ok);
        let data = data.ok().unwrap();
        assert_that!(data, is_some);
        let data = data.unwrap();
        assert_that!(*data, eq 123);
    }

    #[test]
    fn loan_request_is_initialized_with_default_value<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<LifetimeTracker, u64>()
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let tracker = LifetimeTracker::start_tracking();
        let _request = sut.loan();
        assert_that!(tracker.number_of_living_instances(), eq 1);
    }

    #[test]
    fn drop_is_not_called_for_underlying_type_of_requests<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<LifetimeTracker, u64>()
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let tracker = LifetimeTracker::start_tracking();
        let request = sut.loan();
        assert_that!(tracker.number_of_living_instances(), eq 1);
        drop(request);
        assert_that!(tracker.number_of_living_instances(), eq 1);
    }

    #[test]
    fn loan_uninit_request_is_not_initialized<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<LifetimeTracker, u64>()
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let tracker = LifetimeTracker::start_tracking();
        let request = sut.loan_uninit();
        assert_that!(tracker.number_of_living_instances(), eq 0);

        drop(request);
        assert_that!(tracker.number_of_living_instances(), eq 0);
    }

    #[test]
    fn sending_requests_reduces_loan_counter<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(1)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let request = sut.loan().unwrap();

        let request2 = sut.loan();
        assert_that!(request2.err(), eq Some(LoanError::ExceedsMaxLoans));

        request.send().unwrap();

        let request2 = sut.loan();
        assert_that!(request2, is_ok);
    }

    #[test]
    fn dropping_requests_reduces_loan_counter<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(1)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let request = sut.loan().unwrap();

        let request2 = sut.loan();
        assert_that!(request2.err(), eq Some(LoanError::ExceedsMaxLoans));

        drop(request);

        let request2 = sut.loan();
        assert_that!(request2, is_ok);
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
    fn loaned_requests_has_default_constructed_request_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<89123, 98123891>;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_user_header::<UserHeader>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let sut = client.loan().unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn uninitialized_loaned_requests_has_default_constructed_request_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<789123, 798123891>;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_user_header::<UserHeader>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let sut = client.loan_uninit().unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn loaned_slice_requests_has_default_constructed_request_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<9889123, 4598123891>;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        let service = node
            .service_builder(&service_name)
            .request_response::<[u64], u64>()
            .request_user_header::<UserHeader>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let sut = client.loan_slice(1).unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn uninitialized_loaned_slice_requests_has_default_constructed_request_header<Sut: Service>() {
        type UserHeader = CustomUserHeader<5557832, 2341>;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        let service = node
            .service_builder(&service_name)
            .request_response::<[u64], u64>()
            .request_user_header::<UserHeader>()
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let sut = client.loan_slice_uninit(1).unwrap();

        assert_that!(*sut.user_header(), eq UserHeader::default());
    }

    #[test]
    fn request_is_correctly_aligned<Sut: Service>() {
        const MAX_LOAN: usize = 9;
        const ALIGNMENT: usize = 512;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .request_payload_alignment(Alignment::new(ALIGNMENT).unwrap())
            .max_loaned_requests(MAX_LOAN)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let mut requests = vec![];

        for _ in 0..MAX_LOAN {
            let request = sut.loan().unwrap();
            let request_addr = (request.deref() as *const u64) as usize;
            assert_that!(request_addr % ALIGNMENT, eq 0);
            requests.push(request);
        }
    }

    #[test]
    fn send_request_fails_when_already_active_requests_is_at_max<Sut: Service>() {
        const MAX_ACTIVE_REQUESTS: usize = 9;
        const ITERATIONS: usize = 5;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        for _ in 0..ITERATIONS {
            let mut requests = vec![];

            for _ in 0..MAX_ACTIVE_REQUESTS {
                requests.push(sut.send_copy(123).unwrap());
            }

            assert_that!(sut.send_copy(123).err(), eq Some(RequestSendError::ExceedsMaxActiveRequests));

            let request = sut.loan().unwrap();
            assert_that!(request.send().err(), eq Some(RequestSendError::ExceedsMaxActiveRequests));
        }
    }

    fn client_never_goes_out_of_memory_impl<Sut: Service>(
        max_active_requests_per_client: usize,
        max_servers: usize,
        max_loaned_requests: usize,
    ) {
        const ITERATIONS: usize = 5;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(1)
            .max_servers(max_servers)
            .max_active_requests_per_client(max_active_requests_per_client)
            .max_loaned_requests(max_loaned_requests)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let mut servers = vec![];
        for _ in 0..max_servers {
            let sut_server = service.server_builder().create().unwrap();
            servers.push(sut_server);
        }

        for _ in 0..ITERATIONS {
            // max out pending responses
            let mut pending_responses = vec![];
            let mut active_requests = vec![];
            for _ in 0..max_active_requests_per_client {
                pending_responses.push(sut.send_copy(123).unwrap());

                for server in &servers {
                    let active_request = server.receive().unwrap();
                    assert_that!(active_request, is_some);
                    active_requests.push(active_request);
                }
            }

            pending_responses.clear();
            // max out request buffer on server side
            for _ in 0..max_active_requests_per_client {
                pending_responses.push(sut.send_copy(456).unwrap());
            }

            // max out loaned requests
            let mut loaned_requests = vec![];
            for _ in 0..max_loaned_requests {
                let request = sut.loan();
                assert_that!(request, is_ok);
                loaned_requests.push(request);
            }

            let request = sut.loan();
            assert_that!(request.err(), eq Some(LoanError::ExceedsMaxLoans));

            // cleanup
            pending_responses.clear();
            loaned_requests.clear();
            for server in &servers {
                while let Ok(Some(_)) = server.receive() {}
            }
        }
    }

    #[test]
    fn client_never_goes_out_of_memory_with_huge_max_pending_responses<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 100;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn client_never_goes_out_of_memory_with_huge_max_servers<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 100;
        const MAX_LOANED_REQUESTS: usize = 1;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn client_never_goes_out_of_memory_with_huge_max_loaned_requests<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 100;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn client_never_goes_out_of_memory_with_smallest_possible_values<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn client_never_goes_out_of_memory_with_huge_values<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 12;
        const MAX_SERVERS: usize = 15;
        const MAX_LOANED_REQUESTS: usize = 19;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    fn completion_channel_capacity_is_never_exceeded_impl<Sut: Service>(
        max_active_requests_per_client: usize,
        max_servers: usize,
        max_loaned_requests: usize,
    ) {
        const ITERATIONS: usize = 5;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(1)
            .max_servers(max_servers)
            .max_active_requests_per_client(max_active_requests_per_client)
            .max_loaned_requests(max_loaned_requests)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        let mut servers = vec![];
        for _ in 0..max_servers {
            let sut_server = service.server_builder().create().unwrap();
            servers.push(sut_server);
        }

        for _ in 0..ITERATIONS {
            // max out pending responses
            let mut pending_responses = vec![];
            let mut active_requests = vec![];
            for _ in 0..max_active_requests_per_client {
                pending_responses.push(sut.send_copy(123).unwrap());

                for server in &servers {
                    let active_request = server.receive().unwrap();
                    assert_that!(active_request, is_some);
                    active_requests.push(active_request);
                }
            }

            pending_responses.clear();
            // max out request buffer on server side
            for _ in 0..max_active_requests_per_client {
                pending_responses.push(sut.send_copy(456).unwrap());
            }

            // receive and return everything
            active_requests.clear();
            for server in &servers {
                while let Ok(Some(_)) = server.receive() {}
            }
        }
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_active_requests<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 100;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_max_servers<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 100;
        const MAX_LOANED_REQUESTS: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_max_loaned_requests<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 100;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_huge_values<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 23;
        const MAX_SERVERS: usize = 12;
        const MAX_LOANED_REQUESTS: usize = 10;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn completion_channel_capacity_is_never_exceeded_with_smallest_possible_values<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[test]
    fn reclaims_all_requests_delivered_to_server_after_a_server_disconnect<Sut: Service>() {
        const MAX_ACTIVE_REQUESTS: usize = 4;
        const ITERATIONS: usize = 20;
        const MAX_SERVER: usize = 4;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_active_requests_per_client(MAX_ACTIVE_REQUESTS)
            .max_servers(MAX_SERVER)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        for n in 0..MAX_SERVER {
            for _ in 0..ITERATIONS {
                let mut requests = vec![];
                let mut servers = vec![];
                for _ in 0..n {
                    servers.push(service.server_builder().create().unwrap());
                }

                for _ in 0..MAX_ACTIVE_REQUESTS {
                    requests.push(sut.send_copy(123).unwrap());
                }
                assert_that!(sut.send_copy(123).err(), eq Some(RequestSendError::ExceedsMaxActiveRequests));

                // disconnect all servers by dropping them
                drop(servers);
            }
        }
    }

    #[test]
    fn updates_connections_after_reconnect<Sut: Service>() {
        const RECONNECTIONS: usize = 20;
        const MAX_SERVERS: usize = 4;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<usize, u64>()
            .max_servers(MAX_SERVERS)
            .create()
            .unwrap();

        let sut = service.client_builder().create().unwrap();

        for n in 0..MAX_SERVERS {
            for k in 0..RECONNECTIONS {
                let mut servers = vec![];
                for _ in 0..n {
                    servers.push(service.server_builder().create().unwrap());
                }

                for server in &servers {
                    assert_that!(server.has_requests(), eq Ok(false));
                }
                let _pending_response = sut.send_copy(n + k).unwrap();

                for server in &servers {
                    assert_that!(server.has_requests(), eq Ok(true));
                    assert_that!(*server.receive().unwrap().unwrap(), eq n + k);
                }
            }
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
