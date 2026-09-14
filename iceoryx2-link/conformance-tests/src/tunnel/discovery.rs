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
pub mod tunnel_discovery {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::TunnelFixture;
    use crate::parameters::AnyService;
    use crate::testing::{rejecting, retry, service_exists, side};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn a_service_offered_on_one_side_is_bridged_on_both<
        S: Service,
        X: AnyService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // Two sides tunnelled over one carrier, an application on side A
        // offering a service.
        let mut a = side::<S, _>(|config| fixture.tunnel(config));
        let mut b = side::<S, _>(|config| fixture.tunnel(config));
        let service_name = X::service_name();
        let _service = X::create::<S>(&a.node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);

        // === BRIDGE ===
        // Side A bridges and announces, side B mirrors the announcement as
        // a service of the same pattern.
        a.link.discover().expect("discovery succeeds");
        assert_that!(a.link.bridges().contains(&hash), eq true);

        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                match b.link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("the service is not bridged on the opposing side"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is bridged on the opposing side");
        assert_that!(service_exists::<S>(&service_name, &b.config, X::PATTERN), eq true);
    }

    #[conformance_test]
    pub fn two_services_offered_by_one_side_are_both_bridged_on_the_other<
        S: Service,
        X: AnyService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // Two sides tunnelled over one carrier, an application on side A
        // offering two services.
        let mut a = side::<S, _>(|config| fixture.tunnel(config));
        let mut b = side::<S, _>(|config| fixture.tunnel(config));
        let first_name = X::service_name();
        let second_name = X::service_name();
        let _first = X::create::<S>(&a.node, &first_name);
        let _second = X::create::<S>(&a.node, &second_name);
        let first = ServiceHash::new::<S::ServiceNameHasher>(&first_name, X::PATTERN);
        let second = ServiceHash::new::<S::ServiceNameHasher>(&second_name, X::PATTERN);

        // === BRIDGE ===
        // Side A bridges and announces both, side B mirrors both, one
        // peer's offers never displacing each other.
        a.link.discover().expect("discovery succeeds");
        assert_that!(a.link.bridges().contains(&first), eq true);
        assert_that!(a.link.bridges().contains(&second), eq true);

        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                let bridges = b.link.bridges();
                match (bridges.contains(&first), bridges.contains(&second)) {
                    (true, true) => Ok(()),
                    (false, _) => Err("the first service is not bridged on the opposing side"),
                    (_, false) => Err("the second service is not bridged on the opposing side"),
                }
            },
            TIMEOUT,
        )
        .expect("both services are bridged on the opposing side");

        // === HOLD ===
        // Another cycle keeps both.
        b.link.discover().expect("discovery succeeds");
        assert_that!(b.link.bridges().contains(&first), eq true);
        assert_that!(b.link.bridges().contains(&second), eq true);
        assert_that!(service_exists::<S>(&first_name, &b.config, X::PATTERN), eq true);
        assert_that!(service_exists::<S>(&second_name, &b.config, X::PATTERN), eq true);
    }

    #[conformance_test]
    pub fn a_service_the_filter_rejects_is_not_mirrored<
        S: Service,
        X: AnyService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // Two sides tunnelled over one carrier, an application on side A
        // offering a service and side B's filter rejecting it.
        let mut a = side::<S, _>(|config| fixture.tunnel(config));
        let b = side::<S, _>(|config| fixture.tunnel(config));
        let service_name = X::service_name();
        let _service = X::create::<S>(&a.node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);
        let (filter, rejected) = rejecting(service_name);
        let mut b_link = b.link.with_filter(filter);

        // === FILTER ===
        // Side B learns of the service, is asked about its mirror and
        // rejects it.
        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b_link.discover().expect("discovery succeeds");
                match rejected.get() {
                    true => Ok(()),
                    false => Err("the filter has not rejected the service"),
                }
            },
            TIMEOUT,
        )
        .expect("the filter rejects the service");

        assert_that!(b_link.bridges().contains(&hash), eq false);
        assert_that!(service_exists::<S>(&service_name, &b.config, X::PATTERN), eq false);
    }

    #[conformance_test]
    pub fn a_withdrawn_service_is_unbridged_on_both_sides<
        S: Service,
        X: AnyService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // A service offered on side A and mirrored on side B.
        let mut a = side::<S, _>(|config| fixture.tunnel(config));
        let mut b = side::<S, _>(|config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service = X::create::<S>(&a.node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);

        a.link.discover().expect("discovery succeeds");
        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                match b.link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("the service is not bridged on the opposing side"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is bridged on the opposing side");

        // === WITHDRAW ===
        // Side A withdraws. The mirror on side B has no local user, so it
        // must not keep the service alive.
        drop(service);

        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b.link.discover().expect("discovery succeeds");
                match a.link.bridges().empty() && b.link.bridges().empty() {
                    true => Ok(()),
                    false => Err("the service is still bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is unbridged on both sides");
    }

    #[conformance_test]
    pub fn a_withdrawn_service_does_not_survive_through_its_mirrors<
        S: Service,
        X: AnyService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // Side A offers a service, sides B and C mirror it.
        let mut a = side::<S, _>(|config| fixture.tunnel(config));
        let mut b = side::<S, _>(|config| fixture.tunnel(config));
        let mut c = side::<S, _>(|config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service = X::create::<S>(&a.node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);
        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b.link.discover().expect("discovery succeeds");
                c.link.discover().expect("discovery succeeds");
                match b.link.bridges().contains(&hash) && c.link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("the service is not mirrored on both sides"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is mirrored on both sides");

        // === WITHDRAW ===
        // Side A withdraws. A mirror is not a local offer, so neither B nor
        // C keeps the service alive for the other.
        drop(service);

        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b.link.discover().expect("discovery succeeds");
                c.link.discover().expect("discovery succeeds");
                match a.link.bridges().empty()
                    && b.link.bridges().empty()
                    && c.link.bridges().empty()
                {
                    true => Ok(()),
                    false => Err("the service survives on a side"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is gone from every side");
    }

    #[conformance_test]
    pub fn a_mirror_an_application_uses_is_not_offered_back<
        S: Service,
        X: AnyService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();

        // === SETUP ===
        // A service offered on side A and mirrored on side B, where an
        // application opens a port on the mirror.
        let mut a = side::<S, _>(|config| fixture.tunnel(config));
        let mut b = side::<S, _>(|config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service = X::create::<S>(&a.node, &service_name);
        let hash = ServiceHash::new::<S::ServiceNameHasher>(&service_name, X::PATTERN);

        a.link.discover().expect("discovery succeeds");
        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                match b.link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("the service is not bridged on the opposing side"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is bridged on the opposing side");
        let mirror = X::open::<S>(&b.node, &service_name);
        let _subscriber = X::create_port(&mirror);
        b.link.discover().expect("discovery succeeds");

        // === WITHDRAW ===
        // Side A withdraws. The mirror is in use on side B, but it is
        // still the mirror and not an offer of side B's own, so it goes
        // and side A does not mirror it back.
        drop(service);

        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b.link.discover().expect("discovery succeeds");
                match a.link.bridges().empty() && b.link.bridges().empty() {
                    true => Ok(()),
                    false => Err("the service is still bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is unbridged on both sides");
    }
}
