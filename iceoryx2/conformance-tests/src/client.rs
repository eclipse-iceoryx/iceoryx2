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
pub mod client {
    use alloc::{sync::Arc, vec};
    use core::ops::Deref;
    use core::time::Duration;
    use iceoryx2::port::update_connections::UpdateConnections;

    use iceoryx2::port::client::RequestSendError;
    use iceoryx2::port::{LoanError, SendError, UnableToDeliverAction};
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::client::PortFactoryClient;
    use iceoryx2::testing::*;
    use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64, Ordering};
    use iceoryx2_bb_posix::barrier::BarrierBuilder;
    use iceoryx2_bb_posix::barrier::BarrierHandle;
    use iceoryx2_bb_posix::clock::{Time, nanosleep};
    use iceoryx2_bb_posix::ipc_capable::Handle;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing_macros::conformance_test;

    const TIMEOUT: Duration = Duration::from_millis(50);

    fn create_node<Sut: Service>() -> Node<Sut> {
        let config = generate_isolated_config();
        NodeBuilder::new().config(&config).create::<Sut>().unwrap()
    }

    #[conformance_test]
    pub fn disconnected_client_does_not_block_new_clients<Sut: Service>() {
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

    #[conformance_test]
    pub fn send_request_via_disconnected_client_works<Sut: Service>() {
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

    #[conformance_test]
    pub fn can_loan_at_most_max_supported_amount_of_requests<Sut: Service>() {
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

    #[conformance_test]
    pub fn can_loan_max_supported_amount_of_requests_when_holding_max_pending_responses<
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

    #[conformance_test]
    pub fn override_preallocated_requests_to_one_works<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(2)
            .create()
            .unwrap();

        let sut = service
            .client_builder()
            .override_request_preallocation(|_| 1)
            .create()
            .unwrap();

        let _request = sut.loan().unwrap();
        assert_that!(sut.loan().err(), eq Some(LoanError::OutOfMemory));
    }

    #[conformance_test]
    pub fn override_preallocated_requests_to_zero_rounds_up_to_one<Sut: Service>() {
        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_loaned_requests(2)
            .create()
            .unwrap();

        let sut = service
            .client_builder()
            .override_request_preallocation(|_| 0)
            .create()
            .unwrap();

        let _request = sut.loan().unwrap();
        assert_that!(sut.loan().err(), eq Some(LoanError::OutOfMemory));
    }

    #[conformance_test]
    pub fn override_preallocated_requests_to_many_works<Sut: Service>() {
        const MAX_NUMBER_OF_REQUESTS: usize = 10;
        let service_name = generate_service_name();
        let node = create_node::<Sut>();

        for n in 1..MAX_NUMBER_OF_REQUESTS {
            let service = node
                .service_builder(&service_name)
                .request_response::<u64, u64>()
                .max_loaned_requests(n + 1)
                .create()
                .unwrap();

            let number_of_requests = n;
            let sut = service
                .client_builder()
                .override_request_preallocation(move |_| number_of_requests)
                .create()
                .unwrap();

            let mut requests = vec![];
            for _ in 0..n {
                requests.push(sut.loan().unwrap());
            }
            assert_that!(sut.loan().err(), eq Some(LoanError::OutOfMemory));
        }
    }

    #[conformance_test]
    pub fn unable_to_deliver_strategy_block_blocks_when_server_buffer_is_full<Sut: Service>() {
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
        let has_sent_request = AtomicBool::new(false);
        let init_handle = BarrierHandle::new();
        let init_barrier = BarrierBuilder::new(2).create(&init_handle).unwrap();
        let start_handle = BarrierHandle::new();
        let start_barrier = BarrierBuilder::new(2).create(&start_handle).unwrap();

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut = service
                    .client_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::RetryUntilDelivered)
                    .create()
                    .unwrap();

                assert_that!(sut.unable_to_deliver_strategy(), eq UnableToDeliverStrategy::RetryUntilDelivered);

                init_barrier.wait();
                start_barrier.wait();
                let request = sut.send_copy(123);
                assert_that!(request, is_ok);
                drop(request);

                let request = sut.send_copy(123);
                has_sent_request.store(true, Ordering::Relaxed);
                assert_that!(request, is_ok);
            })?;

            init_barrier.wait();
            server.update_connections().unwrap();
            start_barrier.wait();
            nanosleep(TIMEOUT).unwrap();
            assert_that!(has_sent_request.load(Ordering::Relaxed), eq false);
            let data = server.receive();
            assert_that!(data, is_ok);
            assert_that!(|| has_sent_request.load(Ordering::Relaxed), eq true, before Watchdog::default());

            Ok(())
        }).unwrap();
    }

    #[conformance_test]
    pub fn unable_to_deliver_strategy_block_unblocks_when_server_disconnects<Sut: Service>() {
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
        let has_sent_request = AtomicBool::new(false);
        let init_handle = BarrierHandle::new();
        let init_barrier = BarrierBuilder::new(2).create(&init_handle).unwrap();
        let start_handle = BarrierHandle::new();
        let start_barrier = BarrierBuilder::new(2).create(&start_handle).unwrap();

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut = service
                    .client_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::RetryUntilDelivered)
                    .create()
                    .unwrap();

                assert_that!(sut.unable_to_deliver_strategy(), eq UnableToDeliverStrategy::RetryUntilDelivered);

                init_barrier.wait();

                start_barrier.wait();

                sut.send_copy(123).unwrap();
                sut.send_copy(456).unwrap();

                has_sent_request.store(true, Ordering::Relaxed);
            })?;

            init_barrier.wait();
            server.update_connections().unwrap();

            start_barrier.wait();
            nanosleep(TIMEOUT).unwrap();
            assert_that!(has_sent_request.load(Ordering::Relaxed), eq false);

            drop(server);
            assert_that!(|| has_sent_request.load(Ordering::Relaxed), eq true, before Watchdog::default());

            Ok(())
        }).unwrap();
    }

    #[conformance_test]
    pub fn unable_to_deliver_strategy_discard_discards_request<Sut: Service>() {
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
            .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardData)
            .create()
            .unwrap();

        assert_that!(sut.unable_to_deliver_strategy(), eq UnableToDeliverStrategy::DiscardData);

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

    const VALUE_FIRST_REQUEST: u64 = 123;
    const VALUE_SECOND_REQUEST: u64 = 456;

    fn client_with_unable_to_deliver_handler<Sut, ClientBuilder>(
        save_overflow: bool,
        client_builder: ClientBuilder,
        expected_second_send_result: Result<(), RequestSendError>,
        expected_receive_value_server_1: Option<u64>,
        expected_receive_value_server_2: Option<u64>,
    ) -> Duration
    where
        Sut: Service,
        ClientBuilder: Fn(
            PortFactoryClient<'_, Sut, u64, (), u64, ()>,
        ) -> PortFactoryClient<'_, Sut, u64, (), u64, ()>,
    {
        let _watchdog = Watchdog::new();

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .enable_safe_overflow_for_requests(save_overflow)
            .max_active_requests_per_client(1)
            .create()
            .unwrap();

        let server_1 = service.server_builder().create().unwrap();
        let server_2 = service.server_builder().create().unwrap();

        let client_port_factory = service.client_builder();

        let sut = client_builder(client_port_factory).create().unwrap();

        let _ = server_1.update_connections();
        let _ = server_2.update_connections();

        let now = Time::now().unwrap();

        let request = sut.send_copy(VALUE_FIRST_REQUEST);
        assert_that!(request, is_ok);
        assert_that!(
            *server_2.receive().unwrap().unwrap(),
            eq(VALUE_FIRST_REQUEST)
        );

        // need to drop request in order to make the client send the second request,
        // else there will be a RequestSendError::ExceedsMaxActiveRequests error
        drop(request);

        let request = sut.send_copy(VALUE_SECOND_REQUEST);
        match expected_second_send_result {
            Ok(_) => assert_that!(request, is_ok),
            Err(e) => assert_that!(request.err().unwrap(), eq(e)),
        }

        let elapsed_blocking_time = now.elapsed().unwrap();

        // check result for server 1
        let mut receive_result = server_1.receive();
        if let Some(expected_value) = expected_receive_value_server_1 {
            assert_that!(receive_result, is_ok);
            let receive_value = receive_result.unwrap();
            assert_that!(receive_value, is_some);
            let sample = receive_value.unwrap();
            assert_that!(*sample, eq(expected_value));

            receive_result = server_1.receive();
        }
        assert_that!(receive_result, is_ok);
        let receive_value = receive_result.unwrap();
        assert_that!(receive_value, is_none);

        // check result for server 2
        let mut receive_result = server_2.receive();
        if let Some(expected_value) = expected_receive_value_server_2 {
            assert_that!(receive_result, is_ok);
            let receive_value = receive_result.unwrap();
            assert_that!(receive_value, is_some);
            let sample = receive_value.unwrap();
            assert_that!(*sample, eq(expected_value));

            receive_result = server_2.receive();
        }
        assert_that!(receive_result, is_ok);
        let receive_value = receive_result.unwrap();
        assert_that!(receive_value, is_none);

        elapsed_blocking_time
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_does_not_block_with_safe_overflow<Sut: Service>() {
        const SAFE_OVERFLOW: bool = true;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), RequestSendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE_SERVER_1: Option<u64> = Some(VALUE_SECOND_REQUEST);
        const EXPECTED_RECEIVE_VALUE_SERVER_2: Option<u64> = Some(VALUE_SECOND_REQUEST);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        client_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |client_port_factory| {
                client_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |_| {
                        handler_call_count.fetch_add(1, Ordering::Relaxed);
                        UnableToDeliverAction::Retry
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE_SERVER_1,
            EXPECTED_RECEIVE_VALUE_SERVER_2,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(0));
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_discards_request<Sut: Service>() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), RequestSendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE_SERVER_1: Option<u64> = Some(VALUE_FIRST_REQUEST);
        const EXPECTED_RECEIVE_VALUE_SERVER_2: Option<u64> = Some(VALUE_SECOND_REQUEST);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        client_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |client_port_factory| {
                client_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |_| {
                        handler_call_count.fetch_add(1, Ordering::Relaxed);
                        UnableToDeliverAction::DiscardData
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE_SERVER_1,
            EXPECTED_RECEIVE_VALUE_SERVER_2,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(1));
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_retries_twice<Sut: Service>() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), RequestSendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE_SERVER_1: Option<u64> = Some(VALUE_FIRST_REQUEST);
        const EXPECTED_RECEIVE_VALUE_SERVER_2: Option<u64> = Some(VALUE_SECOND_REQUEST);

        let handler_call_count = Arc::new(AtomicU64::new(0));
        const RETRY_COUNT: u64 = 2;

        client_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |client_port_factory| {
                client_port_factory.set_unable_to_deliver_handler({
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
            EXPECTED_RECEIVE_VALUE_SERVER_1,
            EXPECTED_RECEIVE_VALUE_SERVER_2,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(RETRY_COUNT));
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_retries_until_timeout<Sut: Service>() {
        const TIMEOUT: Duration = Duration::from_millis(25);
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), RequestSendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE_SERVER_1: Option<u64> = Some(VALUE_FIRST_REQUEST);
        const EXPECTED_RECEIVE_VALUE_SERVER_2: Option<u64> = Some(VALUE_SECOND_REQUEST);

        let elapsed_blocking_time = client_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |client_port_factory| {
                client_port_factory.set_unable_to_deliver_handler({
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
            EXPECTED_RECEIVE_VALUE_SERVER_1,
            EXPECTED_RECEIVE_VALUE_SERVER_2,
        );

        assert_that!(elapsed_blocking_time, time_at_least(TIMEOUT));
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_discards_request_and_fails<Sut: Service>() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), RequestSendError> =
            Err(RequestSendError::SendError(SendError::UnableToDeliver));
        const EXPECTED_RECEIVE_VALUE_SERVER_1: Option<u64> = Some(VALUE_FIRST_REQUEST);
        const EXPECTED_RECEIVE_VALUE_SERVER_2: Option<u64> = Some(VALUE_SECOND_REQUEST);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        client_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |client_port_factory| {
                client_port_factory.set_unable_to_deliver_handler({
                    let handler_call_count = handler_call_count.clone();
                    move |_| {
                        handler_call_count.fetch_add(1, Ordering::Relaxed);
                        UnableToDeliverAction::DiscardDataAndFail
                    }
                })
            },
            EXPECTED_SECOND_SEND_RESULT,
            EXPECTED_RECEIVE_VALUE_SERVER_1,
            EXPECTED_RECEIVE_VALUE_SERVER_2,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(1));
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_follows_unable_to_deliver_strategy_with_discard_data<
        Sut: Service,
    >() {
        const SAFE_OVERFLOW: bool = false;
        const EXPECTED_SECOND_SEND_RESULT: Result<(), RequestSendError> = Ok(());
        const EXPECTED_RECEIVE_VALUE_SERVER_1: Option<u64> = Some(VALUE_FIRST_REQUEST);
        const EXPECTED_RECEIVE_VALUE_SERVER_2: Option<u64> = Some(VALUE_SECOND_REQUEST);

        let handler_call_count = Arc::new(AtomicU64::new(0));

        client_with_unable_to_deliver_handler::<Sut, _>(
            SAFE_OVERFLOW,
            |client_port_factory| {
                client_port_factory
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
            EXPECTED_RECEIVE_VALUE_SERVER_1,
            EXPECTED_RECEIVE_VALUE_SERVER_2,
        );

        assert_that!(handler_call_count.load(Ordering::Relaxed), eq(1));
    }

    #[conformance_test]
    pub fn client_with_unable_to_deliver_handler_follows_unable_to_deliver_strategy_with_retry_until_delivered<
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
                .enable_safe_overflow_for_requests(false)
                .max_active_requests_per_client(1)
                .create()
                .unwrap();

            let server_1 = service.server_builder().create().unwrap();

            s.thread_builder().spawn(|| {
                let call_count = Arc::new(AtomicU64::new(0));

                let server_2 = service.server_builder().create().unwrap();

                let sut = service
                    .client_builder()
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

                let _ = server_2.update_connections();

                ready_barrier.wait();

                let request = sut.send_copy(123);
                assert_that!(request, is_ok);
                assert_that!(*server_2.receive().unwrap().unwrap(), eq(123));

                // need to drop request in order to make the client send the second request,
                // else there will be a RequestSendError::ExceedsMaxActiveRequests error
                drop(request);

                start_barrier.wait();

                let request = sut.send_copy(456);
                assert_that!(request, is_ok);
                assert_that!(*server_2.receive().unwrap().unwrap(), eq(456));

                finish_barrier.wait();

                assert_that!(call_count.load(Ordering::Relaxed), eq(1));
            })?;

            connect_barrier.wait();

            let _ = server_1.update_connections();

            ready_barrier.wait();

            let start = Time::now().unwrap();

            start_barrier.wait();

            nanosleep(TIMEOUT).unwrap();

            assert_that!(*server_1.receive().unwrap().unwrap(), eq(123));

            finish_barrier.wait();

            assert_that!(*server_1.receive().unwrap().unwrap(), eq(456));

            assert_that!(start.elapsed().unwrap(),  time_at_least TIMEOUT);

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn loan_request_is_initialized_with_default_value<Sut: Service>() {
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

    #[conformance_test]
    pub fn drop_is_not_called_for_underlying_type_of_requests<Sut: Service>() {
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

    #[conformance_test]
    pub fn loan_uninit_request_is_not_initialized<Sut: Service>() {
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

    #[conformance_test]
    pub fn sending_requests_reduces_loan_counter<Sut: Service>() {
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

    #[conformance_test]
    pub fn dropping_requests_reduces_loan_counter<Sut: Service>() {
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

    #[conformance_test]
    pub fn loaned_requests_has_default_constructed_request_header<Sut: Service>() {
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

    #[conformance_test]
    pub fn uninitialized_loaned_requests_has_default_constructed_request_header<Sut: Service>() {
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

    #[conformance_test]
    pub fn loaned_slice_requests_has_default_constructed_request_header<Sut: Service>() {
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

    #[conformance_test]
    pub fn uninitialized_loaned_slice_requests_has_default_constructed_request_header<
        Sut: Service,
    >() {
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

    #[conformance_test]
    pub fn request_is_correctly_aligned<Sut: Service>() {
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

    #[conformance_test]
    pub fn send_request_fails_when_already_active_requests_is_at_max<Sut: Service>() {
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

    #[conformance_test]
    pub fn client_never_goes_out_of_memory_with_huge_max_pending_responses<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 100;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn client_never_goes_out_of_memory_with_huge_max_servers<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 100;
        const MAX_LOANED_REQUESTS: usize = 1;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn client_never_goes_out_of_memory_with_huge_max_loaned_requests<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 100;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn client_never_goes_out_of_memory_with_smallest_possible_values<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        client_never_goes_out_of_memory_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn client_never_goes_out_of_memory_with_huge_values<Sut: Service>() {
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

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_active_requests<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 100;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_max_servers<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 100;
        const MAX_LOANED_REQUESTS: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_max_loaned_requests<
        Sut: Service,
    >() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 100;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_huge_values<Sut: Service>() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 23;
        const MAX_SERVERS: usize = 12;
        const MAX_LOANED_REQUESTS: usize = 10;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn completion_channel_capacity_is_never_exceeded_with_smallest_possible_values<
        Sut: Service,
    >() {
        const MAX_ACTIVE_REQUEST_PER_CLIENT: usize = 1;
        const MAX_SERVERS: usize = 1;
        const MAX_LOANED_REQUESTS: usize = 1;

        completion_channel_capacity_is_never_exceeded_impl::<Sut>(
            MAX_ACTIVE_REQUEST_PER_CLIENT,
            MAX_SERVERS,
            MAX_LOANED_REQUESTS,
        );
    }

    #[conformance_test]
    pub fn reclaims_all_requests_delivered_to_server_after_a_server_disconnect<Sut: Service>() {
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

    #[conformance_test]
    pub fn updates_connections_after_reconnect<Sut: Service>() {
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
}
