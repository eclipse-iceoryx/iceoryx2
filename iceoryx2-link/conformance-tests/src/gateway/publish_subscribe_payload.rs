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
pub mod gateway_publish_subscribe_payload {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::{DiscoverableEndpoints, GatewayFixture, PayloadEndpoints};

    use crate::parameters::{Header, PayloadShape, PublishSubscribeService, Value};
    use crate::testing::{Side, describe, retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn payloads_from_services_without_headers_flow_in_both_directions<
        S: Service,
        X: PublishSubscribeService,
        F: GatewayFixture<S, RemoteEndpoints: DiscoverableEndpoints + PayloadEndpoints<Value<X>>>,
    >() {
        let from_remote = X::Payload::value(7);
        let from_local = X::Payload::value(9);

        let mut fixture = F::new();

        // === SETUP ===
        // A local application's service without a header, bridged by the
        // link, and remote endpoints joining where it maps to.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(|_| fixture.gateway());
        let service_name = X::service_name();
        let service = X::create_service::<(), _>(&node, &service_name);
        let publisher = X::create_publisher(&service);
        let subscriber = X::create_subscriber(&service);
        let description = describe::<S, X::Payload, ()>(&service_name, &config);
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
        let remote = fixture.remote_endpoints_on(&description);
        assert_that!(remote.sync(TIMEOUT), eq true);

        // === REMOTE TO LOCAL ===
        remote.send_payload(from_remote.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive(&subscriber) {
                    Some((_, payload)) if payload == from_remote => Ok(()),
                    Some(_) => Err("an unexpected sample arrived locally"),
                    None => Err("no sample arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the message reaches the local application");

        // === LOCAL TO REMOTE ===
        X::send(&publisher, (), from_local.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_payload() {
                    Some(payload) if payload == from_local => Ok(()),
                    Some(_) => Err("an unexpected message arrived remotely"),
                    None => Err("no message arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the sample reaches the remote endpoints");
    }

    #[conformance_test]
    pub fn payloads_from_services_with_headers_flow_in_both_directions<
        S: Service,
        X: PublishSubscribeService,
        F: GatewayFixture<S, RemoteEndpoints: DiscoverableEndpoints + PayloadEndpoints<Value<X>>>,
    >() {
        let from_remote = X::Payload::value(7);
        let from_local = X::Payload::value(9);

        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints on a service with a user header, mirrored by
        // the link, and a local application joining the mirror.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(|_| fixture.gateway());
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
        let subscriber = X::create_subscriber(&service);

        // === REMOTE TO LOCAL ===
        remote.send_payload(from_remote.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match X::receive(&subscriber) {
                    Some((_, payload)) if payload == from_remote => Ok(()),
                    Some(_) => Err("an unexpected sample arrived locally"),
                    None => Err("no sample arrived locally"),
                }
            },
            TIMEOUT,
        )
        .expect("the message reaches the local application");

        // === LOCAL TO REMOTE ===
        X::send(&publisher, Header::<X>::default(), from_local.clone());
        retry(
            || {
                link.propagate().expect("propagation succeeds");
                match remote.receive_payload() {
                    Some(payload) if payload == from_local => Ok(()),
                    Some(_) => Err("an unexpected message arrived remotely"),
                    None => Err("no message arrived remotely"),
                }
            },
            TIMEOUT,
        )
        .expect("the sample reaches the remote endpoints");
    }
}
