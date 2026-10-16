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

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod gateway_request_response {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link::Link;
    use iceoryx2_link_backend::Backend;

    use crate::fixture::{
        Discoverable, GatewayFixture, PropagatesRequestResponse, ResponseMessage,
    };
    use crate::parameters::{
        PayloadShape, RequestHeader, RequestResponseService, RequestValue, ResponseHeader,
        ResponseValue,
    };
    use crate::testing::{Side, description_of, retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    /// Propagates until the remote endpoints receive a request carrying
    /// `expected`, returning the id they gave it.
    fn received_request_id<S, B, R, Q, P>(
        link: &mut Link<S, B>,
        remote: &R,
        expected: &Q,
    ) -> R::RequestId
    where
        S: Service,
        B: Backend<S>,
        R: PropagatesRequestResponse<Q, P>,
        Q: PartialEq,
    {
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_request() {
                    Some((id, request)) if request == *expected => Ok(id),
                    Some(_) => Err("an unexpected request arrived remotely"),
                    None => Err("no request arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the request reaches the remote endpoints")
    }

    /// Propagates until the remote endpoints receive `expected` on the
    /// response stream of `id`.
    fn expect_response<S, B, R, Q, P>(
        link: &mut Link<S, B>,
        remote: &R,
        id: R::RequestId,
        expected: ResponseMessage<P>,
    ) where
        S: Service,
        B: Backend<S>,
        R: PropagatesRequestResponse<Q, P>,
        P: PartialEq,
    {
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_response() {
                    Some((received_id, message)) if received_id == id && message == expected => {
                        Ok(())
                    }
                    Some(_) => Err("an unexpected message arrived remotely"),
                    None => Err("no message arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the message reaches the remote endpoints");
    }

    #[conformance_test]
    pub fn requests_from_local_clients_are_answered_by_remote_servers<
        S: Service,
        X: RequestResponseService,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponse<RequestValue<X>, ResponseValue<X>>,
            >,
    >() {
        let request = X::Request::value(7);
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
        let pending = X::send_request(&client, RequestHeader::<X>::default(), request.clone());
        let id = received_request_id(&mut link, &remote, &request);

        // === RESPOND ===
        remote.send_response(id, response.clone());
        remote.end_response_stream(id);
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_response(&pending) {
                    Some((_, received)) if received == response => Ok(()),
                    Some(_) => Err("an unexpected response arrived locally"),
                    None => Err("no response arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the response reaches the local client");
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match pending.is_connected() {
                    false => Ok(()),
                    true => Err("the response stream has not ended"),
                }
            },
            TIMEOUT,
        )
        .expect("the response stream ends for the local client");
    }

    #[conformance_test]
    pub fn requests_from_remote_clients_are_answered_by_local_servers<
        S: Service,
        X: RequestResponseService,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponse<RequestValue<X>, ResponseValue<X>>,
            >,
    >() {
        let request = X::Request::value(7);
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
        let id = remote.send_request(request.clone());
        let active = retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_request(&server) {
                    Some(received) if X::request_value(&received) == request => Ok(received),
                    Some(_) => Err("an unexpected request arrived locally"),
                    None => Err("no request arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the request reaches the local server");

        // === RESPOND ===
        // The server answers and lets go of the request, ending its stream.
        X::send_response(&active, ResponseHeader::<X>::default(), response.clone());
        drop(active);
        expect_response(&mut link, &remote, id, ResponseMessage::Response(response));
        expect_response(&mut link, &remote, id, ResponseMessage::End);

        // The injected request does not come back to the remote endpoints.
        assert_that!(remote.receive_request(), is_none);
    }

    #[conformance_test]
    pub fn responses_reach_the_client_that_asked_with_several_outstanding<
        S: Service,
        X: RequestResponseService,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponse<RequestValue<X>, ResponseValue<X>>,
            >,
    >() {
        let first_request = X::Request::value(1);
        let second_request = X::Request::value(2);
        let first_response = X::Response::value(3);
        let second_response = X::Response::value(4);

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
        // Two requests outstanding at once.
        let first = X::send_request(
            &client,
            RequestHeader::<X>::default(),
            first_request.clone(),
        );
        let second = X::send_request(
            &client,
            RequestHeader::<X>::default(),
            second_request.clone(),
        );
        let first_id = received_request_id(&mut link, &remote, &first_request);
        let second_id = received_request_id(&mut link, &remote, &second_request);

        // === RESPOND ===
        // Answered in the opposite order, each response reaches its
        // request.
        remote.send_response(second_id, second_response.clone());
        remote.send_response(first_id, first_response.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_response(&second) {
                    Some((_, received)) if received == second_response => Ok(()),
                    Some(_) => Err("an unexpected response arrived for the second request"),
                    None => Err("no response arrived for the second request"),
                }
            },
            TIMEOUT,
        )
        .expect("the response reaches the second request");
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_response(&first) {
                    Some((_, received)) if received == first_response => Ok(()),
                    Some(_) => Err("an unexpected response arrived for the first request"),
                    None => Err("no response arrived for the first request"),
                }
            },
            TIMEOUT,
        )
        .expect("the response reaches the first request");
    }

    #[conformance_test]
    pub fn fire_and_forget_requests_reach_remote_servers<
        S: Service,
        X: RequestResponseService,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponse<RequestValue<X>, ResponseValue<X>>,
            >,
    >() {
        let request = X::Request::value(7);

        let mut fixture = F::new();

        // === SETUP ===
        // A local application's fire-and-forget service with a client,
        // bridged by the link, and remote endpoints joining where it maps
        // to.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(fixture.config(), |_| fixture.gateway());
        let service_name = X::service_name();
        let service = X::create_service_with::<RequestHeader<X>, ResponseHeader<X>, S>(
            &node,
            &service_name,
            |builder| builder.enable_fire_and_forget_requests(true),
        );
        let client = X::create_client(&service);
        let description = description_of::<S>(&service_name, &config, X::PATTERN);
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
        // The client lets go of the request as soon as it is sent.
        drop(X::send_request(
            &client,
            RequestHeader::<X>::default(),
            request.clone(),
        ));
        received_request_id(&mut link, &remote, &request);
    }

    #[conformance_test]
    pub fn responses_to_abandoned_requests_are_discarded<
        S: Service,
        X: RequestResponseService,
        F: GatewayFixture<
                S,
                RemoteEndpoints: Discoverable
                                     + PropagatesRequestResponse<RequestValue<X>, ResponseValue<X>>,
            >,
    >() {
        let abandoned_request = X::Request::value(7);
        let stale_response = X::Response::value(8);
        let request = X::Request::value(9);
        let response = X::Response::value(10);

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

        // === ABANDON ===
        // The client lets go of a request the remote endpoints received,
        // then they answer it.
        let abandoned = X::send_request(
            &client,
            RequestHeader::<X>::default(),
            abandoned_request.clone(),
        );
        let abandoned_id = received_request_id(&mut link, &remote, &abandoned_request);
        drop(abandoned);
        link.propagate().expect("propagation succeeds");
        remote.send_response(abandoned_id, stale_response);
        remote.end_response_stream(abandoned_id);

        // === REQUEST ===
        // The stale response is discarded and a new request round-trips.
        let pending = X::send_request(&client, RequestHeader::<X>::default(), request.clone());
        let id = received_request_id(&mut link, &remote, &request);
        remote.send_response(id, response.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive_response(&pending) {
                    Some((_, received)) if received == response => Ok(()),
                    Some(_) => Err("an unexpected response arrived locally"),
                    None => Err("no response arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the response reaches the local client");
    }
}
