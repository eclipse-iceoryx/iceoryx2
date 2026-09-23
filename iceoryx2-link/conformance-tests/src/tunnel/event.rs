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
pub mod tunnel_event {
    use core::time::Duration;

    use iceoryx2::port::event_id::EventId;
    use iceoryx2::service::Service;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use crate::fixture::TunnelFixture;
    use crate::parameters::EventService;
    use crate::testing::{retry, side};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const EVENT_ID_A: usize = 3;
    const EVENT_ID_B: usize = 4;

    #[conformance_test]
    pub fn notifications_flow_in_both_directions<S: Service, X: EventService, F: TunnelFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // An event service on side A, bridged on both sides, a notifier
        // and a listener on each side.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service_a = X::create_service::<S>(&a.node, &service_name);
        let notifier_a = X::create_notifier(&service_a);
        let listener_a = X::create_listener(&service_a);
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
        let service_b = X::open_service::<S>(&b.node, &service_name);
        let notifier_b = X::create_notifier(&service_b);
        let listener_b = X::create_listener(&service_b);
        assert_that!(fixture.sync(&hash, TIMEOUT), eq true);

        // === A TO B ===
        // Side A's own listener hears the notification at once, as any
        // listener on the service does.
        X::notify(&notifier_a, EventId::new(EVENT_ID_A));
        assert_that!(X::notifications(&listener_a), eq alloc::vec![EventId::new(EVENT_ID_A)]);
        retry(
            || {
                a.link.propagate().expect("propagation succeeds");
                b.link.propagate().expect("propagation succeeds");
                match X::notifications(&listener_b).as_slice() {
                    [id] if *id == EventId::new(EVENT_ID_A) => Ok(()),
                    [] => Err("no notification arrived on side B"),
                    _ => Err("an unexpected notification arrived on side B"),
                }
            },
            TIMEOUT,
        )
        .expect("the notification reaches side B");

        // === B TO A ===
        X::notify(&notifier_b, EventId::new(EVENT_ID_B));
        assert_that!(X::notifications(&listener_b), eq alloc::vec![EventId::new(EVENT_ID_B)]);
        retry(
            || {
                b.link.propagate().expect("propagation succeeds");
                a.link.propagate().expect("propagation succeeds");
                match X::notifications(&listener_a).as_slice() {
                    [id] if *id == EventId::new(EVENT_ID_B) => Ok(()),
                    [] => Err("no notification arrived on side A"),
                    _ => Err("an unexpected notification arrived on side A"),
                }
            },
            TIMEOUT,
        )
        .expect("the notification reaches side A");

        // === NO ECHO ===
        // Neither side's link hands a relayed notification back, so no
        // listener hears a notification a second time.
        for _ in 0..3 {
            a.link.propagate().expect("propagation succeeds");
            b.link.propagate().expect("propagation succeeds");
        }
        assert_that!(X::notifications(&listener_a), len 0);
        assert_that!(X::notifications(&listener_b), len 0);
    }
}
