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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod server {
    use alloc::{sync::Arc, vec};
    use core::time::Duration;
    use iceoryx2::port::update_connections::UpdateConnections;

    use iceoryx2::port::{ReceiveError, SendError, UnableToDeliverAction};
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::service::port_factory::server::PortFactoryServer;
    use iceoryx2::testing::*;
    use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64, Ordering};
    use iceoryx2_bb_posix::barrier::BarrierBuilder;
    use iceoryx2_bb_posix::barrier::BarrierHandle;
    use iceoryx2_bb_posix::clock::{Time, nanosleep};
    use iceoryx2_bb_posix::ipc_capable::Handle;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing_macros::conformance_test;

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

    #[conformance_test]
    pub fn disconnected_server_does_not_block_new_servers<Sut: Service>() {
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

    #[conformance_test]
    pub fn receiving_requests_works_with_server_created_first<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();
        let sut = service.server_builder().create().unwrap();
        let client = service.client_builder().create().unwrap();

        assert_that!(sut.has_requests(), eq Ok(false));
        let _pending_response = client.send_copy(1234).unwrap();
        assert_that!(sut.has_requests(), eq Ok(true));

        let active_request = sut.receive().unwrap().unwrap();
        assert_that!(*active_request, eq 1234);
    }

    #[conformance_test]
    pub fn receiving_requests_works_with_client_created_first<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();
        let client = service.client_builder().create().unwrap();
        let sut = service.server_builder().create().unwrap();

        assert_that!(sut.has_requests(), eq Ok(false));
        let _pending_response = client.send_copy(5678);
        assert_that!(sut.has_requests(), eq Ok(true));

        let active_request = sut.receive().unwrap().unwrap();
        assert_that!(*active_request, eq 5678);
    }

    #[conformance_test]
    pub fn requests_of_a_disconnected_client_are_not_received<Sut: Service>() {
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

    #[conformance_test]
    pub fn requests_are_not_delivered_to_late_joiners<Sut: Service>() {
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

    #[conformance_test]
    pub fn max_loaned_responses_per_requests_is_adjusted_to_sane_values<Sut: Service>() {
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

    #[conformance_test]
    pub fn can_loan_at_most_max_responses_per_requests<Sut: Service>() {
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

    #[conformance_test]
    pub fn override_preallocated_responses_to_one_works<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let sut = service
            .server_builder()
            .override_response_preallocation(|_| 1)
            .max_loaned_responses_per_request(2)
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let _pending_response = client.send_copy(0).unwrap();

        let active_request = sut.receive().unwrap().unwrap();

        let _response = active_request.loan().unwrap();
        assert_that!(active_request.loan().err(), eq Some(iceoryx2::port::LoanError::OutOfMemory));
    }

    pub fn override_preallocated_responses_to_zero_rounds_up_to_one<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .create()
            .unwrap();

        let sut = service
            .server_builder()
            .override_response_preallocation(|_| 0)
            .max_loaned_responses_per_request(2)
            .create()
            .unwrap();

        let client = service.client_builder().create().unwrap();
        let _pending_response = client.send_copy(0).unwrap();

        let active_request = sut.receive().unwrap().unwrap();

        let _response = active_request.loan().unwrap();
        assert_that!(active_request.loan().err(), eq Some(iceoryx2::port::LoanError::OutOfMemory));
    }

    pub fn override_preallocated_responses_many_works<Sut: Service>() {
        const MAX_NUMBER_OF_RESPONSES: usize = 10;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        for n in 1..MAX_NUMBER_OF_RESPONSES {
            let service = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .create()
                .unwrap();

            let number_of_responses = n;
            let sut = service
                .server_builder()
                .override_response_preallocation(move |_| number_of_responses)
                .max_loaned_responses_per_request(n + 1)
                .create()
                .unwrap();

            let client = service.client_builder().create().unwrap();
            let _pending_response = client.send_copy(0).unwrap();

            let active_request = sut.receive().unwrap().unwrap();

            let mut responses = vec![];
            for _ in 0..n {
                responses.push(active_request.loan().unwrap());
            }

            assert_that!(active_request.loan().err(), eq Some(iceoryx2::port::LoanError::OutOfMemory));
        }
    }

    #[conformance_test]
    pub fn server_can_hold_specified_amount_of_active_requests_one_client_one_request<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[conformance_test]
    pub fn server_can_hold_specified_amount_of_active_requests_one_client_many_requests<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 1;
        const MAX_ACTIVE_REQUESTS: usize = 9;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[conformance_test]
    pub fn server_can_hold_specified_amount_of_active_requests_many_clients_one_request<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 7;
        const MAX_ACTIVE_REQUESTS: usize = 1;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[conformance_test]
    pub fn server_can_hold_specified_amount_of_active_requests_many_clients_many_requests<
        Sut: Service,
    >() {
        const MAX_CLIENTS: usize = 8;
        const MAX_ACTIVE_REQUESTS: usize = 9;
        server_can_hold_specified_amount_of_active_requests::<Sut>(
            MAX_ACTIVE_REQUESTS,
            MAX_CLIENTS,
        );
    }

    #[conformance_test]
    pub fn unable_to_deliver_strategy_discard_discards_responses_when_client_buffer_is_full<
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
            .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardData)
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

    #[conformance_test]
    pub fn unable_to_deliver_strategy_block_blocks_responses_when_client_buffer_is_full<
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
        let handle = BarrierHandle::new();
        let send_handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
        let send_barrier = BarrierBuilder::new(2).create(&send_handle).unwrap();

        let has_sent_response = AtomicBool::new(false);
        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut = service
                    .server_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::RetryUntilDelivered)
                    .create()
                    .unwrap();
                barrier.wait();
                send_barrier.wait();

                let active_request = sut.receive().unwrap().unwrap();
                assert_that!(active_request.send_copy(321), is_ok);

                assert_that!(active_request.send_copy(654), is_ok);
                has_sent_response.store(true, Ordering::Relaxed);
            })?;

            barrier.wait();
            let pending_response = client.send_copy(123).unwrap();
            send_barrier.wait();

            nanosleep(TIMEOUT).unwrap();

            assert_that!(has_sent_response.load(Ordering::Relaxed), eq false);
            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response, eq 321);
            assert_that!(|| has_sent_response.load(Ordering::Relaxed), eq true, before Watchdog::default());

            let response = pending_response.receive().unwrap().unwrap();
            assert_that!(*response, eq 654);

            Ok(())
        }).unwrap();
    }

    #[conformance_test]
    pub fn unable_to_deliver_strategy_block_unblocks_when_pending_response_goes_out_of_scope<
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
        let handle = BarrierHandle::new();
        let send_handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
        let send_barrier = BarrierBuilder::new(2).create(&send_handle).unwrap();

        let has_sent_response = AtomicBool::new(false);
        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut = service
                    .server_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::RetryUntilDelivered)
                    .create()
                    .unwrap();
                barrier.wait();
                send_barrier.wait();

                let active_request = sut.receive().unwrap().unwrap();
                assert_that!(active_request.send_copy(321), is_ok);
                assert_that!(active_request.send_copy(654), is_ok);
                has_sent_response.store(true, Ordering::Relaxed);
            })?;

            barrier.wait();
            let pending_response = client.send_copy(123).unwrap();
            send_barrier.wait();

            nanosleep(TIMEOUT).unwrap();

            assert_that!(has_sent_response.load(Ordering::Relaxed), eq false);
            drop(pending_response);

            Ok(())
        })
        .unwrap();
    }

    const VALUE_FIRST_RESPONSE: u64 = 123;
    const VALUE_SECOND_RESPONSE: u64 = 456;

    fn server_with_unable_to_deliver_handler<Sut, ServerBuilder>(
        save_overflow: bool,
        server_builder: ServerBuilder,
        expected_second_send_result: Result<(), SendError>,
        expected_receive_value: Option<u64>,
    ) -> Duration
    where
        Sut: Service,
        ServerBuilder: Fn(
            PortFactoryServer<'_, Sut, u64, (), u64, ()>,
        ) -> PortFactoryServer<'_, Sut, u64, (), u64, ()>,
    {
        let _watchdog = Watchdog::new();

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_response_buffer_size(1)
            .enable_safe_overflow_for_responses(save_overflow)
            .create()
            .unwrap();

        let server_port_factory = service.server_builder();

        let sut = server_builder(server_port_factory).create().unwrap();

        let client = service.client_builder().create().unwrap();

        let _ = sut.update_connections();

        let now = Time::now().unwrap();

        let pending_response = client.send_copy(13).unwrap();

        let active_request = sut.receive().unwrap().unwrap();

        assert_that!(active_request.send_copy(VALUE_FIRST_RESPONSE), eq(Ok(())));
        assert_that!(
            active_request.send_copy(VALUE_SECOND_RESPONSE),
            eq(expected_second_send_result)
        );

        let elapsed_blocking_time = now.elapsed().unwrap();

        // check receive result
        let mut receive_result = pending_response.receive();
        if let Some(expected_value) = expected_receive_value {
            assert_that!(receive_result, is_ok);
            let receive_value = receive_result.unwrap();
            assert_that!(receive_value, is_some);
            let sample = receive_value.unwrap();
            assert_that!(*sample, eq(expected_value));

            receive_result = pending_response.receive();
        }
        assert_that!(receive_result, is_ok);
        let receive_value = receive_result.unwrap();
        assert_that!(receive_value, is_none);

        elapsed_blocking_time
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_does_not_block_with_safe_overflow<Sut: Service>() {
        const SAFE_OVERFLOW: bool = true;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), SendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE: Option<u64> = Some(VALUE_SECOND_RESPONSE);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        server_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |server_port_factory| {
                server_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |_| {
                        handler_call_count.fetch_add(1, Ordering::Relaxed);
                        UnableToDeliverAction::Retry
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(0));
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_discards_response<Sut: Service>() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), SendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE: Option<u64> = Some(VALUE_FIRST_RESPONSE);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        server_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |server_port_factory| {
                server_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |_| {
                        handler_call_count.fetch_add(1, Ordering::Relaxed);
                        UnableToDeliverAction::DiscardData
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(1));
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_retries_twice<Sut: Service>() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), SendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE: Option<u64> = Some(VALUE_FIRST_RESPONSE);

        let handler_call_count = Arc::new(AtomicU64::new(0));
        const RETRY_COUNT: u64 = 2;

        server_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |server_port_factory| {
                server_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |info| {
                        if info.retries == RETRY_COUNT {
                            UnableToDeliverAction::DiscardData
                        } else {
                            handler_call_count.fetch_add(1, Ordering::Relaxed);
                            UnableToDeliverAction::Retry
                        }
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(RETRY_COUNT));
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_retries_until_timeout<Sut: Service>() {
        const TIMEOUT: Duration = Duration::from_millis(25);
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), SendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE: Option<u64> = Some(VALUE_FIRST_RESPONSE);

        let elapsed_blocking_time = server_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |server_port_factory| {
                server_port_factory.set_unable_to_deliver_handler({
                    move |info| {
                        if info.elapsed_time > TIMEOUT {
                            UnableToDeliverAction::DiscardData
                        } else {
                            UnableToDeliverAction::Retry
                        }
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE,
        );

        assert_that!(elapsed_blocking_time, time_at_least(TIMEOUT));
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_discards_response_and_fails<Sut: Service>() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), SendError> = Err(SendError::UnableToDeliver);
        const EXPECTED_RECEIVE_VALUE: Option<u64> = Some(VALUE_FIRST_RESPONSE);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        server_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |server_port_factory| {
                server_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |_| {
                        handler_call_count.fetch_add(1, Ordering::Relaxed);
                        UnableToDeliverAction::DiscardDataAndFail
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(1));
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_follows_unable_to_deliver_strategy_with_discard_data<
        Sut: Service,
    >() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), SendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE: Option<u64> = Some(VALUE_FIRST_RESPONSE);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        server_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |server_port_factory| {
                server_port_factory
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardData)
                    .set_unable_to_deliver_handler({
                        let handler_call_count = handler_call_count.clone();
                        move |_| {
                            handler_call_count.fetch_add(1, Ordering::Relaxed);
                            UnableToDeliverAction::FollowUnableToDeliveryStrategy
                        }
                    })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(1));
    }

    #[conformance_test]
    pub fn server_with_unable_to_deliver_handler_follows_unable_to_deliver_strategy_with_retry_until_delivered<
        Sut: Service,
    >() {
        let _watchdog = Watchdog::new();

        thread_scope(|s| {
            const TIMEOUT: Duration = Duration::from_millis(25);

            let connect_handle = BarrierHandle::new();
            let connect_barrier = BarrierBuilder::new(2).create(&connect_handle).unwrap();
            let ready_handle = BarrierHandle::new();
            let ready_barrier = BarrierBuilder::new(2).create(&ready_handle).unwrap();
            let start_handle = BarrierHandle::new();
            let start_barrier = BarrierBuilder::new(2).create(&start_handle).unwrap();
            let finish_handle = BarrierHandle::new();
            let finish_barrier = BarrierBuilder::new(2).create(&finish_handle).unwrap();

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

            s.thread_builder().spawn(|| {
                let call_count = Arc::new(AtomicU64::new(0));

                let sut = service
                    .server_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::RetryUntilDelivered)
                    .set_unable_to_deliver_handler({
                        let call_count = call_count.clone();
                        move |_| {
                            call_count.fetch_add(1, Ordering::Relaxed);
                            UnableToDeliverAction::FollowUnableToDeliveryStrategy
                        }
                    })
                    .create()
                    .unwrap();

                connect_barrier.wait();

                let _ = sut.update_connections();

                ready_barrier.wait();
                start_barrier.wait();

                let active_request = sut.receive().unwrap().unwrap();

                assert_that!(active_request.send_copy(1), is_ok);
                assert_that!(active_request.send_copy(2), is_ok);

                finish_barrier.wait();

                assert_that!(call_count.load(Ordering::Relaxed), eq(1));
            })?;

            connect_barrier.wait();
            ready_barrier.wait();

            let pending_response = client.send_copy(123).unwrap();

            let start = Time::now().unwrap();

            start_barrier.wait();

            nanosleep(TIMEOUT).unwrap();

            assert_that!(*pending_response.receive().unwrap().unwrap(), eq 1);

            finish_barrier.wait();

            assert_that!(*pending_response.receive().unwrap().unwrap(), eq 2);

            assert_that!(start.elapsed().unwrap(),  time_at_least TIMEOUT);

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn reclaims_all_responses_delivered_to_client_after_a_client_disconnect<Sut: Service>() {
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

    #[conformance_test]
    pub fn updates_connections_after_reconnect<Sut: Service>() {
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

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_buffer_size<Sut: Service>() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 100;
        const MAX_BORROWED_RESPONSES: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_response_borrow_size<
        Sut: Service,
    >() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 1;
        const MAX_BORROWED_RESPONSES: usize = 100;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_values<Sut: Service>() {
        const MAX_RESPONSE_BUFFER_SIZE: usize = 100;
        const MAX_BORROWED_RESPONSES: usize = 100;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_RESPONSE_BUFFER_SIZE,
            MAX_BORROWED_RESPONSES,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_smallest_possible_values<
        Sut: Service,
    >() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_smallest_possible_values<Sut: Service>() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_huge_values<Sut: Service>() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_huge_max_clients<Sut: Service>() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_huge_max_active_requests<Sut: Service>() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_huge_response_buffer_size<Sut: Service>() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_huge_borrowed_responses<Sut: Service>() {
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

    #[conformance_test]
    pub fn server_runs_never_out_of_memory_with_huge_max_loaned_responses<Sut: Service>() {
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

    #[conformance_test]
    pub fn can_loan_per_request_at_most_number_of_responses<Sut: Service>() {
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
}
