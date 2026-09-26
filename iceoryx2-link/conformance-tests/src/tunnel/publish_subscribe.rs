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
pub mod tunnel_publish_subscribe {
    use core::time::Duration;

    use iceoryx2::service::Service;
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_backend::service_description::PublishSubscribeSettings;

    use crate::fixture::TunnelFixture;
    use crate::parameters::Foreign as Y;
    use crate::parameters::{PayloadShape, PublishSubscribeService};
    use crate::testing::{
        hash_of, history_size_of, notifications_of, payload_type_name, payload_type_of, retry, side,
    };

    const TIMEOUT: Duration = Duration::from_secs(10);
    const SAMPLE_A: u64 = 1;
    const SAMPLE_B: u64 = 2;
    const PROPAGATIONS: usize = 10;

    #[conformance_test]
    pub fn a_mirror_has_the_local_default_settings<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        const HISTORY_SIZE: usize = 7;

        let mut fixture = F::new();

        // === SETUP ===
        // Side B offers a service with a history size of its own.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let service_name = X::service_name();
        let _service = X::create_service_with::<(), _>(&b.node, &service_name, |builder| {
            builder.history_size(HISTORY_SIZE)
        });
        let defaults = PublishSubscribeSettings::from_config(&a.config);

        // === MIRROR ===
        // Side A mirrors it with side A's defaults, settings are local.
        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                a.link.discover().expect("discovery succeeds");
                match history_size_of::<S>(&service_name, &a.config) {
                    Some(size) if size == defaults.history_size => Ok(()),
                    Some(_) => Err("the mirror has other settings than the local defaults"),
                    None => Err("the service is not mirrored"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is mirrored with the local defaults");
    }

    #[conformance_test]
    pub fn a_service_recreated_with_other_types_replaces_its_mirror<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        let mut fixture = F::new();
        let payload = payload_type_name::<X>();
        let foreign_payload = payload_type_name::<Y>();

        // === SETUP ===
        // Side B offers a service, side A mirrors it with B's types.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service = X::create_service::<(), _>(&b.node, &service_name);
        let hash = hash_of::<S>(&service_name);
        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                a.link.discover().expect("discovery succeeds");
                match payload_type_of::<S>(&service_name, &a.config) {
                    Some(name) if name == payload => Ok(()),
                    Some(_) => Err("the mirror has other types"),
                    None => Err("the service is not mirrored"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is mirrored with its types");

        // === RECREATE ===
        // Side B withdraws and, once the mirror is gone, recreates the
        // service with other types. Side A mirrors it anew with those.
        drop(service);
        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                a.link.discover().expect("discovery succeeds");
                match a.link.bridges().contains(&hash) {
                    false => Ok(()),
                    true => Err("the mirror is still bridged"),
                }
            },
            TIMEOUT,
        )
        .expect("the mirror is gone");
        let _service = Y::create_service::<(), _>(&b.node, &service_name);

        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                a.link.discover().expect("discovery succeeds");
                match payload_type_of::<S>(&service_name, &a.config) {
                    Some(name) if name == foreign_payload => Ok(()),
                    Some(_) => Err("the mirror has the old types"),
                    None => Err("the service is not mirrored"),
                }
            },
            TIMEOUT,
        )
        .expect("the service is mirrored with its new types");
    }

    #[conformance_test]
    pub fn samples_flow_in_both_directions<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        let sample_a = X::Payload::value(SAMPLE_A);
        let sample_b = X::Payload::value(SAMPLE_B);
        let mut fixture = F::new();

        // === SETUP ===
        // A service offered on side A and mirrored on side B, with an
        // application on each side holding a publisher and a subscriber.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service_a = X::create_service::<(), _>(&a.node, &service_name);
        let hash = hash_of::<S>(&service_name);
        let publisher_a = X::create_publisher(&service_a);
        let subscriber_a = X::create_subscriber(&service_a);

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

        // === JOIN ===
        // Side B's application joins the mirror the link created.
        let service_b = X::open_service::<(), _>(&b.node, &service_name);
        let publisher_b = X::create_publisher(&service_b);
        let subscriber_b = X::create_subscriber(&service_b);
        assert_that!(fixture.sync(&hash, TIMEOUT), eq true);

        // === A TO B ===
        X::send(&publisher_a, (), sample_a.clone());
        retry(
            || {
                a.link.propagate().expect("propagation succeeds");
                b.link.propagate().expect("propagation succeeds");
                match X::receive(&subscriber_b) {
                    Some((_, payload)) if payload == sample_a => Ok(()),
                    Some(_) => Err("an unexpected sample arrived on side B"),
                    None => Err("no sample arrived on side B"),
                }
            },
            TIMEOUT,
        )
        .expect("the sample reaches side B");

        // === B TO A ===
        X::send(&publisher_b, (), sample_b.clone());
        retry(
            || {
                b.link.propagate().expect("propagation succeeds");
                a.link.propagate().expect("propagation succeeds");
                loop {
                    match X::receive(&subscriber_a) {
                        Some((_, payload)) if payload == sample_b => return Ok(()),
                        // Side A's own publication is seen by its own subscriber.
                        Some((_, payload)) if payload == sample_a => continue,
                        Some(_) => return Err("an unexpected sample arrived on side A"),
                        None => return Err("no sample arrived on side A"),
                    }
                }
            },
            TIMEOUT,
        )
        .expect("the sample reaches side A");
        assert_that!(a.link.bridges().contains(&hash), eq true);
    }

    #[conformance_test]
    pub fn samples_flow_between_services_with_differing_settings<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        const HISTORY_SIZE_A: usize = 1;
        const HISTORY_SIZE_B: usize = 2;
        const SAMPLE: u64 = 5;
        let sample = X::Payload::value(SAMPLE);

        let mut fixture = F::new();

        // === SETUP ===
        // Each side's application creates the service itself, with the same
        // types and its own history size. Neither side mirrors, each
        // exports its own.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service_a = X::create_service_with::<(), _>(&a.node, &service_name, |builder| {
            builder.history_size(HISTORY_SIZE_A)
        });
        let service_b = X::create_service_with::<(), _>(&b.node, &service_name, |builder| {
            builder.history_size(HISTORY_SIZE_B)
        });
        let hash = hash_of::<S>(&service_name);
        let publisher_a = X::create_publisher(&service_a);
        let subscriber_b = X::create_subscriber(&service_b);
        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b.link.discover().expect("discovery succeeds");
                match a.link.bridges().contains(&hash) && b.link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("a side has not bridged its service"),
                }
            },
            TIMEOUT,
        )
        .expect("both sides bridge their own service");
        assert_that!(fixture.sync(&hash, TIMEOUT), eq true);

        // === A TO B ===
        // Settings are local, the types are what the channel is keyed by.
        X::send(&publisher_a, (), sample.clone());
        retry(
            || {
                a.link.propagate().expect("propagation succeeds");
                b.link.propagate().expect("propagation succeeds");
                match X::receive(&subscriber_b) {
                    Some((_, payload)) if payload == sample => Ok(()),
                    Some(_) => Err("an unexpected sample arrived on side B"),
                    None => Err("no sample arrived on side B"),
                }
            },
            TIMEOUT,
        )
        .expect("the sample reaches side B");
    }

    #[conformance_test]
    pub fn samples_do_not_cross_between_services_with_differing_types<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        const SAMPLE: u64 = 5;
        let sample = X::Payload::value(SAMPLE);

        let mut fixture = F::new();

        // === SETUP ===
        // Each side's application creates the service itself, with other
        // types. Both are bridged, on different channels.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let service_name = X::service_name();
        let service_a = X::create_service::<(), _>(&a.node, &service_name);
        let service_b = Y::create_service::<(), _>(&b.node, &service_name);
        let hash = hash_of::<S>(&service_name);
        let publisher_a = X::create_publisher(&service_a);
        let subscriber_b = Y::create_subscriber(&service_b);
        retry(
            || {
                a.link.discover().expect("discovery succeeds");
                b.link.discover().expect("discovery succeeds");
                match a.link.bridges().contains(&hash) && b.link.bridges().contains(&hash) {
                    true => Ok(()),
                    false => Err("a side has not bridged its service"),
                }
            },
            TIMEOUT,
        )
        .expect("both sides bridge their own service");

        // === A TO B ===
        // Side a sends bytes of a type different from the side B so should
        // never be received.
        X::send(&publisher_a, (), sample.clone());
        for _ in 0..20 {
            a.link.propagate().expect("propagation succeeds");
            b.link.propagate().expect("propagation succeeds");
        }
        assert_that!(Y::receive(&subscriber_b), is_none);
    }

    #[conformance_test]
    pub fn a_delivery_notifies_the_event_service_named_after_the_service<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        let sample = X::Payload::value(SAMPLE_A);
        let mut fixture = F::new();

        // === SETUP ===
        // A service offered on side A and mirrored on side B, whose link
        // notifies on delivery.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        b.link = b.link.with_notifications();

        let service_name = X::service_name();
        let service_a = X::create_service::<(), _>(&a.node, &service_name);
        let publisher_a = X::create_publisher(&service_a);
        let hash = hash_of::<S>(&service_name);

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

        // === JOIN ===
        // Side B's application subscribes to the mirror and listens on the
        // event service named after it.
        let service_b = X::open_service::<(), _>(&b.node, &service_name);
        let subscriber_b = X::create_subscriber(&service_b);

        let event_b = b
            .node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .expect("the event service is created");
        let listener_b = event_b
            .listener_builder()
            .create()
            .expect("the listener is created");

        assert_that!(fixture.sync(&hash, TIMEOUT), eq true);

        // === A TO B ===
        // The sample delivered on side B is announced on the event service.
        X::send(&publisher_a, (), sample.clone());

        retry(
            || {
                a.link.propagate().expect("propagation succeeds");
                b.link.propagate().expect("propagation succeeds");
                match notifications_of(&listener_b).is_empty() {
                    false => Ok(()),
                    true => Err("the event service was not notified"),
                }
            },
            TIMEOUT,
        )
        .expect("the delivery is notified");

        match X::receive(&subscriber_b) {
            Some((_, payload)) => assert_that!(payload, eq sample),
            None => panic!("the notified sample is not available"),
        }
    }

    #[conformance_test]
    pub fn a_delivery_notification_does_not_cross_to_the_opposing_side<
        S: Service,
        X: PublishSubscribeService,
        F: TunnelFixture,
    >() {
        let sample = X::Payload::value(SAMPLE_A);
        let mut fixture = F::new();

        // === SETUP ===
        // A service and the event service named after it offered on side A,
        // both mirrored on side B, whose link notifies on delivery.
        let mut a = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        let mut b = side::<S, _>(fixture.config(), |config| fixture.tunnel(config));
        b.link = b.link.with_notifications();

        let service_name = X::service_name();
        let service_a = X::create_service::<(), _>(&a.node, &service_name);
        let publisher_a = X::create_publisher(&service_a);
        let hash = hash_of::<S>(&service_name);

        let event_a = a
            .node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .expect("the event service is created");
        let listener_a = event_a
            .listener_builder()
            .create()
            .expect("the listener is created");
        let event_hash =
            ServiceHash::new::<S::ServiceNameHasher>(&service_name, MessagingPattern::Event);

        a.link.discover().expect("discovery succeeds");
        retry(
            || {
                b.link.discover().expect("discovery succeeds");
                match b.link.bridges().contains(&hash) && b.link.bridges().contains(&event_hash) {
                    true => Ok(()),
                    false => Err("the services are not bridged on the opposing side"),
                }
            },
            TIMEOUT,
        )
        .expect("the services are bridged on the opposing side");

        // === JOIN ===
        // Side B's application listens on the mirrored event service.
        let event_b = b
            .node
            .service_builder(&service_name)
            .event()
            .open()
            .expect("the event service is opened");
        let listener_b = event_b
            .listener_builder()
            .create()
            .expect("the listener is created");

        assert_that!(fixture.sync(&hash, TIMEOUT), eq true);
        assert_that!(fixture.sync(&event_hash, TIMEOUT), eq true);

        // === A TO B ===
        // The delivery notifies side B only. The link does not relay its own
        // notification back to side A.
        X::send(&publisher_a, (), sample);

        retry(
            || {
                a.link.propagate().expect("propagation succeeds");
                b.link.propagate().expect("propagation succeeds");
                match notifications_of(&listener_b).is_empty() {
                    false => Ok(()),
                    true => Err("the event service was not notified"),
                }
            },
            TIMEOUT,
        )
        .expect("the delivery is notified on side B");

        for _ in 0..PROPAGATIONS {
            b.link.propagate().expect("propagation succeeds");
            a.link.propagate().expect("propagation succeeds");
        }
        assert_that!(notifications_of(&listener_a), is_empty);
    }
}
