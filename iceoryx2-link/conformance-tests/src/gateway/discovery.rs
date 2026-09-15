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
pub mod gateway_discovery {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::{DiscoverableEndpoints, GatewayFixture};

    use crate::parameters::AnyService;
    use crate::testing::{Side, rejecting, retry, service_exists, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn a_remote_endpoint_is_mirrored<
        S: Service,
        X: AnyService,
        F: GatewayFixture<S, RemoteEndpoints: DiscoverableEndpoints>,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // A link with nothing offered locally, and remote endpoints on
        // the middleware.
        let Side {
            config, mut link, ..
        } = side::<S, _>(|_| fixture.gateway());
        let remote = fixture.remote_endpoints_on(&X::describe::<S>(&X::service_name(), &config));

        // === MIRROR ===
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
    }

    #[conformance_test]
    pub fn a_remote_endpoint_the_filter_rejects_is_not_mirrored<
        S: Service,
        X: AnyService,
        F: GatewayFixture<S, RemoteEndpoints: DiscoverableEndpoints>,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints on the middleware, and a link whose filter
        // rejects their service.
        let Side { config, link, .. } = side::<S, _>(|_| fixture.gateway());
        let service_name = X::service_name();
        let remote = fixture.remote_endpoints_on(&X::describe::<S>(&service_name, &config));
        let (filter, rejected) = rejecting(service_name);
        let mut link = link.with_filter(filter);

        // === FILTER ===
        // The link learns of the remote endpoints, is asked about their
        // mirror and rejects it.
        retry(
            || {
                link.discover().expect("discovery succeeds");
                match rejected.get() {
                    true => Ok(()),
                    false => Err("the filter has not rejected the service"),
                }
            },
            TIMEOUT,
        )
        .expect("the filter rejects the service");

        assert_that!(link.bridges().contains(&remote.service().hash()), eq false);
        assert_that!(service_exists::<S>(&service_name, &config, X::PATTERN), eq false);
    }

    #[conformance_test]
    pub fn a_withdrawn_remote_endpoint_is_unbridged<
        S: Service,
        X: AnyService,
        F: GatewayFixture<S, RemoteEndpoints: DiscoverableEndpoints>,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // A remote endpoint mirrored by the link.
        let Side {
            config, mut link, ..
        } = side::<S, _>(|_| fixture.gateway());
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

        // === WITHDRAW ===
        // The remote endpoints leave. The gateway's own endpoints remain
        // on the middleware and must not keep the mirror alive.
        drop(remote);

        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().empty() {
                    true => Ok(()),
                    false => Err("the mirror is still bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the mirror is unbridged");
    }
}
