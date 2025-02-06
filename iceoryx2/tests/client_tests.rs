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
    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;

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

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
