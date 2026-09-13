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
pub mod link_discovery {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::LinkFixture;

    use crate::parameters::AnyService;
    use crate::testing::{Side, rejecting, retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn a_local_service_is_bridged<S: Service, X: AnyService, F: LinkFixture<S>>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One link and one application in the same local system.
        let Side { node, mut link, .. } = side::<S, _>(|config| fixture.backend(config));

        let service_name = X::service_name();
        let _service = X::create::<S>(&node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);

        // === BRIDGE ===
        // The application's service is picked up on the first cycle.
        link.discover().expect("discovery succeeds");

        assert_that!(link.bridges().contains(&hash), eq true);
    }

    #[conformance_test]
    pub fn a_local_service_the_filter_rejects_is_not_bridged<
        S: Service,
        X: AnyService,
        F: LinkFixture<S>,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // One application in a local system, and a link whose filter
        // rejects its service.
        let Side { node, link, .. } = side::<S, _>(|config| fixture.backend(config));

        let service_name = X::service_name();
        let _service = X::create::<S>(&node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);
        let (filter, rejected) = rejecting(service_name);
        let mut link = link.with_filter(filter);

        // === FILTER ===
        // The filter is asked about the application's service and rejects it.
        link.discover().expect("discovery succeeds");

        assert_that!(rejected.get(), eq true);
        assert_that!(link.bridges().contains(&hash), eq false);
    }

    #[conformance_test]
    pub fn a_withdrawn_local_service_is_unbridged<S: Service, X: AnyService, F: LinkFixture<S>>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One link and one application whose service is bridged.
        let Side { node, mut link, .. } = side::<S, _>(|config| fixture.backend(config));

        let service_name = X::service_name();
        let service = X::create::<S>(&node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);
        link.discover().expect("discovery succeeds");
        assert_that!(link.bridges().contains(&hash), eq true);

        // === WITHDRAW ===
        // Only the link's own endpoints remain on the service, which does not
        // count as a local offer.
        drop(service);

        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().empty() {
                    true => Ok(()),
                    false => Err("the service is still bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is unbridged");
    }

    #[conformance_test]
    pub fn a_failed_bridge_is_retried<S: Service, X: AnyService, F: LinkFixture<S>>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One link and one application holding every port slot its service
        // allows, so the link cannot open its ports.
        let Side {
            config,
            node,
            mut link,
        } = side::<S, _>(|config| fixture.backend(config));

        let service_name = X::service_name();
        let service = X::create::<S>(&node, &service_name);
        let mut ports = X::occupy_ports::<S>(&service, &config);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);

        // === FAIL ===
        link.discover().expect("discovery succeeds");
        assert_that!(link.bridges().contains(&hash), eq false);

        // === RETRY ===
        // The application frees a slot and the next cycle bridges.
        drop(ports.pop());
        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("the service is not bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the bridge is retried");
    }

    #[conformance_test]
    pub fn a_recreated_local_service_is_bridged_anew<
        S: Service,
        X: AnyService,
        F: LinkFixture<S>,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // One link and one application whose service is bridged.
        let Side { node, mut link, .. } = side::<S, _>(|config| fixture.backend(config));

        let service_name = X::service_name();
        let service = X::create::<S>(&node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);
        link.discover().expect("discovery succeeds");
        assert_that!(link.bridges().contains(&hash), eq true);

        // === RECREATE ===
        // The application withdraws and, once the bridge is gone, creates
        // the service again. It is bridged anew.
        drop(service);
        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().contains(&hash) {
                    false => Ok(()),
                    true => Err("the service is still bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is unbridged");
        let _service = X::create::<S>(&node, &service_name);

        retry(
            || {
                link.discover().expect("discovery succeeds");
                match link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("the service is not bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is bridged anew");
    }
}
