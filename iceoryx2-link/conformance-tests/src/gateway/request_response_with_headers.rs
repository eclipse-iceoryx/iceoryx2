// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

/// The user headers of requests and responses crossing a middleware that
/// carries them intact. Opted into by a fixture whose remote is a
/// [`PropagatesRequestResponseWithHeaders`].
///
/// [`PropagatesRequestResponseWithHeaders`]: crate::fixture::PropagatesRequestResponseWithHeaders
#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod gateway_request_response_with_headers {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::{
        Discoverable, GatewayFixture, PropagatesRequestResponseWithHeaders, ResponseMessage,
    };
    use crate::parameters::{
        PayloadShape, RequestHeader, RequestResponseService, RequestValue, ResponseHeader,
        ResponseValue,
    };
    use crate::testing::{Side, retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn the_headers_reach_the_remote_endpoints_and_the_local_client<
        S: Service,
        X: RequestResponseService<RequestHeader: From<u64>, ResponseHeader: From<u64>>,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponseWithHeaders<
                    RequestHeader<X>,
                    RequestValue<X>,
                    ResponseHeader<X>,
                    ResponseValue<X>,
                >,
            >,
    >() {
        let request_header = RequestHeader::<X>::from(3);
        let request = X::Request::value(7);
        let response_header = ResponseHeader::<X>::from(5);
        let response = X::Response::value(9);

        let mut fixture = F::new();

        // === SETUP ===
        // A local application's service with a client, bridged by the
        // link, and remote endpoints joining where it maps to.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(fixture.config(), |_| fixture.gateway());
        let service_name = X::service_name();
        let service =
            X::create_service::<RequestHeader<X>, ResponseHeader<X>, S>(&node, &service_name);
        let client = X::create_client(&service);
        let description = X::describe::<S>(&service_name, &config);
        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().contains(&description.hash()) {
                    true => Ok(()),
                    false => Err("the service is not bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is bridged");
        // Remote endpoints join the endpoint the service is exported to.
        let remote = fixture.remote_endpoints_on(&description);
        assert_that!(remote.sync(TIMEOUT), eq true);

        // The link learns of the remote endpoints serving the service.
        link.discover().expect("discovery succeeds");

        // === REQUEST ===
        let pending = X::send_request(&client, request_header, request.clone());
        let id = retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_request_with_header() {
                    Some((id, header, received))
                        if header == request_header && received == request =>
                    {
                        Ok(id)
                    }
                    Some(_) => Err("an unexpected request arrived remotely"),
                    None => Err("no request arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the header and the request reach the remote endpoints");

        // === RESPOND ===
        remote.send_response_with_header(id, response_header, response.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_response(&pending) {
                    Some((header, received))
                        if header == response_header && received == response =>
                    {
                        Ok(())
                    }
                    Some(_) => Err("an unexpected response arrived locally"),
                    None => Err("no response arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the header and the response reach the local client");
    }

    #[conformance_test]
    pub fn the_headers_reach_the_local_server_and_the_remote_endpoints<
        S: Service,
        X: RequestResponseService<RequestHeader: From<u64>, ResponseHeader: From<u64>>,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponseWithHeaders<
                    RequestHeader<X>,
                    RequestValue<X>,
                    ResponseHeader<X>,
                    ResponseValue<X>,
                >,
            >,
    >() {
        let request_header = RequestHeader::<X>::from(3);
        let request = X::Request::value(7);
        let response_header = ResponseHeader::<X>::from(5);
        let response = X::Response::value(9);

        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints mirrored by the link, and a local application
        // serving the mirror.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(fixture.config(), |_| fixture.gateway());
        let remote = fixture.remote_endpoints_on(&X::describe::<S>(&X::service_name(), &config));
        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().contains(&remote.service().hash()) {
                    true => Ok(()),
                    false => Err("the remote endpoint is not mirrored"),
                }
            },
            TIMEOUT,
        )
        .expect("the remote endpoint is mirrored");
        assert_that!(remote.sync(TIMEOUT), eq true);
        let service = X::open_service::<RequestHeader<X>, ResponseHeader<X>, S>(
            &node,
            &remote.service().name(),
        );
        let server = X::create_server(&service);

        // === REQUEST ===
        let id = remote.send_request_with_header(request_header, request.clone());
        let active = retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_request(&server) {
                    Some(received)
                        if X::request_header(&received) == request_header
                            && X::request_value(&received) == request =>
                    {
                        Ok(received)
                    }
                    Some(_) => Err("an unexpected request arrived locally"),
                    None => Err("no request arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the header and the request reach the local server");

        // === RESPOND ===
        X::send_response(&active, response_header, response.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_response_with_header() {
                    Some((received_id, ResponseMessage::Response((header, received))))
                        if received_id == id
                            && header == response_header
                            && received == response =>
                    {
                        Ok(())
                    }
                    Some(_) => Err("an unexpected message arrived remotely"),
                    None => Err("no message arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the header and the response reach the remote endpoints");
    }
}
