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

/// The user header crossing a middleware that carries it intact. Opted
/// into by a fixture whose remote is a [`SampleEndpoints`].
///
/// [`SampleEndpoints`]: crate::fixture::SampleEndpoints
#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod gateway_publish_subscribe_header {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::{DiscoverableEndpoints, GatewayFixture, SampleEndpoints};

    use crate::parameters::{Header, PayloadShape, PublishSubscribeService, Value};
    use crate::testing::{Side, describe, retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn the_header_reaches_the_local_application<
        S: Service,
        X: PublishSubscribeService<Header: From<u64>>,
        F: GatewayFixture<
                S,
                RemoteEndpoints: DiscoverableEndpoints + SampleEndpoints<Header<X>, Value<X>>,
            >,
    >() {
        let header = Header::<X>::from(3);
        let payload = X::Payload::value(7);

        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints on a service with a user header, mirrored by
        // the link, and a local application joining the mirror.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(fixture.config(), |_| fixture.gateway());
        let remote = fixture.remote_endpoints_on(&describe::<S, X::Payload, Header<X>>(
            &X::service_name(),
            &config,
        ));
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
        let service = X::open_service::<Header<X>, _>(&node, &remote.service().name());
        let subscriber = X::create_subscriber(&service);

        // === REMOTE TO LOCAL ===
        remote.send_sample(header, payload.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive(&subscriber) {
                    Some((received_header, received_payload))
                        if received_header == header && received_payload == payload =>
                    {
                        Ok(())
                    }
                    Some(_) => Err("an unexpected sample arrived locally"),
                    None => Err("no sample arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the header and the payload reach the local application");
    }

    #[conformance_test]
    pub fn the_header_reaches_the_remote_endpoints<
        S: Service,
        X: PublishSubscribeService<Header: From<u64>>,
        F: GatewayFixture<
                S,
                RemoteEndpoints: DiscoverableEndpoints + SampleEndpoints<Header<X>, Value<X>>,
            >,
    >() {
        let header = Header::<X>::from(3);
        let payload = X::Payload::value(7);

        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints on a service with a user header, mirrored by
        // the link, and a local application joining the mirror.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(fixture.config(), |_| fixture.gateway());
        let remote = fixture.remote_endpoints_on(&describe::<S, X::Payload, Header<X>>(
            &X::service_name(),
            &config,
        ));
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
        let service = X::open_service::<Header<X>, _>(&node, &remote.service().name());
        let publisher = X::create_publisher(&service);

        // === LOCAL TO REMOTE ===
        X::send(&publisher, header, payload.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_sample() {
                    Some((received_header, received_payload))
                        if received_header == header && received_payload == payload =>
                    {
                        Ok(())
                    }
                    Some(_) => Err("an unexpected message arrived remotely"),
                    None => Err("no message arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the header and the payload reach the remote endpoints");
    }
}
