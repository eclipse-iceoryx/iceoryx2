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

    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

    const TIMEOUT: Duration = Duration::from_millis(50);

    // TODO:
    //   - config: add max_borrows_per_pending_response
    //     - add tests for config adjustments etc.
    //   - never goes out of memory
    //     - vary all possibilities
    //   - completion channel capacity is never exceeded
    //     - vary all possibilities
    //   - disconnected server does not block new server
    //   - test max_active_requests
    //   - requests of disconnected client are not received
    //   - reclaims all requests after disconnect
    //  fn concurrent_communication_with_subscriber_reconnects_does_not_deadlock
    //   - comm with max clients/server
    //   - dropping service keeps established comm
    //   - service can be open when there is a server/client
    //
    //   - server
    //     - has requests
    //   - service builder
    //     - ports of dropped service block new service creation
    //     - service can be opened when there is a port
    //     - ?server decrease buffer size? (increase with failure)
    //     - create max amount of ports
    //     - create max amount of nodes

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
    fn loan_and_send_request_works<Sut: Service>() {
        const PAYLOAD: u64 = 2873421;
        let (_node, service) = create_node_and_service::<Sut>();

        let sut = service.client_builder().create().unwrap();
        let mut request = sut.loan().unwrap();
        *request = PAYLOAD;

        let pending_response = request.send();
        assert_that!(pending_response, is_ok);
        let pending_response = pending_response.unwrap();
        assert_that!(*pending_response.payload(), eq PAYLOAD);
    }

    #[test]
    fn can_loan_at_most_max_supported_amount_of_requests<Sut: Service>() {
        const MAX_LOANED_REQUESTS: usize = 29;
        const ITERATIONS: usize = 3;
        let (_node, service) = create_node_and_service::<Sut>();

        let sut = service
            .client_builder()
            .max_loaned_requests(MAX_LOANED_REQUESTS)
            .create()
            .unwrap();

        for _ in 0..ITERATIONS {
            let mut requests = vec![];
            for _ in 0..MAX_LOANED_REQUESTS {
                let request = sut.loan_uninit();
                assert_that!(request, is_ok);
                requests.push(request);
            }
            let request = sut.loan_uninit();
            assert_that!(request.err(), eq Some(LoanError::ExceedsMaxLoanedSamples));
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
            .create()
            .unwrap();
        let server = service.server_builder().buffer_size(1).create().unwrap();
        let has_sent_request = IoxAtomicBool::new(false);
        let barrier = Barrier::new(2);

        std::thread::scope(|s| {
            s.spawn(|| {
                let sut = service
                    .client_builder()
                    .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
                    .create()
                    .unwrap();

                let request = sut.send_copy(123);
                assert_that!(request, is_ok);
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
            .create()
            .unwrap();
        let server = service.server_builder().buffer_size(1).create().unwrap();

        let sut = service
            .client_builder()
            .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
            .create()
            .unwrap();

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
        let request = sut.loan();
        assert_that!(tracker.number_of_living_instances(), eq 1);

        drop(request);
        assert_that!(tracker.number_of_living_instances(), eq 0);
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
        let (_node, service) = create_node_and_service::<Sut>();

        let sut = service
            .client_builder()
            .max_loaned_requests(1)
            .create()
            .unwrap();

        let request = sut.loan().unwrap();

        let request2 = sut.loan();
        assert_that!(request2.err(), eq Some(LoanError::ExceedsMaxLoanedSamples));

        request.send().unwrap();

        let request2 = sut.loan();
        assert_that!(request2, is_ok);
    }

    #[test]
    fn dropping_requests_reduces_loan_counter<Sut: Service>() {
        let (_node, service) = create_node_and_service::<Sut>();

        let sut = service
            .client_builder()
            .max_loaned_requests(1)
            .create()
            .unwrap();

        let request = sut.loan().unwrap();

        let request2 = sut.loan();
        assert_that!(request2.err(), eq Some(LoanError::ExceedsMaxLoanedSamples));

        drop(request);

        let request2 = sut.loan();
        assert_that!(request2, is_ok);
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
            .create()
            .unwrap();

        let sut = service
            .client_builder()
            .max_loaned_requests(MAX_LOAN)
            .create()
            .unwrap();

        let mut requests = vec![];

        for _ in 0..MAX_LOAN {
            let request = sut.loan().unwrap();
            let request_addr = (request.deref() as *const u64) as usize;
            assert_that!(request_addr % ALIGNMENT, eq 0);
            requests.push(request);
        }
    }

    fn client_never_goes_out_of_memory_impl<Sut: Service>(
        max_loaned_requests: usize,
        max_servers: usize,
        max_active_requests: usize,
        max_pending_responses: usize,
        max_request_buffer_size: usize,
    ) {
        const ITERATIONS: usize = 5;

        let service_name = generate_service_name();
        let node = create_node::<Sut>();
        let service = node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .max_clients(1)
            .max_servers(max_servers)
            .max_pending_responses(max_pending_responses)
            .max_active_requests(max_active_requests)
            .max_request_buffer_size(max_request_buffer_size)
            .create()
            .unwrap();

        let sut = service
            .client_builder()
            .max_loaned_requests(max_loaned_requests)
            .create()
            .unwrap();

        let mut servers = vec![];
        for _ in 0..max_servers {
            let sut_server = service.server_builder().create().unwrap();
            servers.push(sut_server);
        }

        for _ in 0..ITERATIONS {
            // max out borrow samples
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
