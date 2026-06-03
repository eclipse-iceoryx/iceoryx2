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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod event_propagation {
    use alloc::string::ToString;
    use core::fmt::Debug;
    use core::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2::testing::generate_service_name;
    use iceoryx2_bb_posix::clock::nanosleep;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_services_tunnel::Tunnel;
    use iceoryx2_services_tunnel_backend::traits::{Backend, testing::Testing};

    fn propagate_events<S: Service, B: Backend<S> + Debug, T: Testing>(num: usize) {
        const TIMEOUT: Duration = Duration::from_millis(250);
        const MAX_ATTEMPTS: usize = 25;

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let iceoryx_config_a = generate_isolated_config();

        let mut tunnel_a = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_a.clone())
            .polled()
            .create()
            .unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let notifier_a = service_a.notifier_builder().create().unwrap();

        tunnel_a.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_hash()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_b.clone())
            .polled()
            .create()
            .unwrap();

        // Wait for tunnel on host b to discover the service on host A
        T::retry(
            || {
                tunnel_b.discover_over_backend().unwrap();
                let service_discovered = tunnel_b.tunneled_services().len() == 1;
                if service_discovered {
                    return Ok(());
                }
                Err("Failed to discover remote services")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();
        T::sync(service_a.service_hash().as_str().to_string(), TIMEOUT);

        // Create a listener to connect to the discovered service
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let listener_b = service_b.listener_builder().create().unwrap();

        // === TEST ===
        for _ in 0..num {
            notifier_a.notify().unwrap();

            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();

            // Receive with retry
            T::retry(
                || match listener_b.try_wait(|_| {}).unwrap() {
                    0 => {
                        tunnel_a.propagate().unwrap();
                        tunnel_b.propagate().unwrap();
                        Err("Failed to receive expected event")
                    }
                    _ => Ok(()),
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            )
            .unwrap();
        }
    }

    #[conformance_test]
    pub fn propagates_event<S: Service, B: Backend<S> + Debug, T: Testing>() {
        propagate_events::<S, B, T>(1);
    }

    #[conformance_test]
    pub fn propagates_event_many<S: Service, B: Backend<S> + Debug, T: Testing>() {
        propagate_events::<S, B, T>(10);
    }

    #[conformance_test]
    pub fn propagated_events_do_not_loop_back<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let iceoryx_config_a = generate_isolated_config();

        let mut tunnel_a = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_a.clone())
            .polled()
            .create()
            .unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let notifier_a = service_a.notifier_builder().create().unwrap();
        let listener_a = service_a.listener_builder().create().unwrap();

        tunnel_a.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_hash()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();

        let mut tunnel_b = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_b.clone())
            .polled()
            .create()
            .unwrap();
        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        T::retry(
            || {
                tunnel_b.discover_over_backend().unwrap();

                let service_discovered = tunnel_b.tunneled_services().len() == 1;
                if service_discovered {
                    return Ok(());
                }

                Err("failed to discover remote service")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();

        T::sync(service_a.service_hash().as_str().to_string(), TIMEOUT);

        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let listener_b = service_b.listener_builder().create().unwrap();

        // === TEST ===
        // Send notification
        notifier_a.notify().unwrap();

        // Drain the notification at host a
        listener_a.try_wait(|_| {}).unwrap();

        // Propagate over tunnels
        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Receive at listener b with retry
        T::retry(
            || match listener_b.try_wait(|_| {}).unwrap() {
                0 => {
                    tunnel_a.propagate().unwrap();
                    tunnel_b.propagate().unwrap();
                    Err("failed to receive expected event")
                }
                _ => Ok(()),
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();

        // Propagate a few times to see if there is a loop-back
        for _ in 0..5 {
            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();
            nanosleep(Duration::from_millis(100)).unwrap();
        }

        // Notification should not have looped back from b to a
        let number_of_notifications = listener_a.try_wait(|_| {}).unwrap();
        assert_that!(number_of_notifications, eq 0);
    }

    // TODO: Fix flaky
    #[conformance_test]
    pub fn multiple_events_are_consolidated_by_id<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let iceoryx_config_a = generate_isolated_config();

        let mut tunnel_a = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_a.clone())
            .polled()
            .create()
            .unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let notifier_a = service_a.notifier_builder().create().unwrap();

        tunnel_a.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_hash()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();

        let mut tunnel_b = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_b.clone())
            .polled()
            .create()
            .unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        T::retry(
            || {
                tunnel_b.discover_over_backend().unwrap();

                let service_discovered = tunnel_b.tunneled_services().len() == 1;
                if service_discovered {
                    return Ok(());
                }

                Err("failed to discover remote service")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();

        T::sync(service_a.service_hash().as_str().to_string(), TIMEOUT);

        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let listener_b = service_b.listener_builder().create().unwrap();

        // === TEST ===
        // Send multiple notifications on different event ids
        let event_a = EventId::new(42);
        let event_b = EventId::new(73);
        let event_c = EventId::new(127);

        const NUM_NOTIFICATIONS: usize = 10;
        for _ in 0..NUM_NOTIFICATIONS {
            notifier_a.notify_with_custom_event_id(event_a).unwrap();
            notifier_a.notify_with_custom_event_id(event_b).unwrap();
            notifier_a.notify_with_custom_event_id(event_c).unwrap();
        }

        // Propagate over tunnels
        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Receive with retry
        let mut num_notifications_a = 0;
        let mut num_notifications_b = 0;
        let mut num_notifications_c = 0;

        T::retry(
            || {
                listener_b
                    .try_wait(|event| {
                        if event.id == event_a {
                            num_notifications_a += event.count;
                        }
                        if event.id == event_b {
                            num_notifications_b += event.count;
                        }
                        if event.id == event_c {
                            num_notifications_c += event.count;
                        }
                    })
                    .unwrap();
                if num_notifications_a == 0 || num_notifications_b == 0 || num_notifications_c == 0
                {
                    tunnel_a.propagate().unwrap();
                    tunnel_b.propagate().unwrap();
                    return Err("expected notifications did not arrive");
                }
                Ok(())
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap();

        assert_that!(num_notifications_a, eq 1);
        assert_that!(num_notifications_b, eq 1);
        assert_that!(num_notifications_c, eq 1);
    }

    #[conformance_test]
    pub fn events_are_routed_to_their_own_service<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const TIMEOUT: Duration = Duration::from_millis(250);
        const MAX_ATTEMPTS: usize = 25;

        // === SETUP ===
        let service_name_1 = generate_service_name();
        let service_name_2 = generate_service_name();

        // --- Host A: two services, one notifier each ---
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_a.clone())
            .polled()
            .create()
            .unwrap();

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();

        let service_a1 = node_a
            .service_builder(&service_name_1)
            .event()
            .open_or_create()
            .unwrap();
        let notifier_a1 = service_a1.notifier_builder().create().unwrap();

        let service_a2 = node_a
            .service_builder(&service_name_2)
            .event()
            .open_or_create()
            .unwrap();
        let notifier_a2 = service_a2.notifier_builder().create().unwrap();

        tunnel_a.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 2);
        assert_that!(tunnel_a.tunneled_services().contains(service_a1.service_hash()), eq true);
        assert_that!(tunnel_a.tunneled_services().contains(service_a2.service_hash()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config_b.clone())
            .polled()
            .create()
            .unwrap();

        // Wait for tunnel on host B to discover both services on host A
        T::retry(
            || {
                tunnel_b.discover_over_backend().unwrap();
                if tunnel_b.tunneled_services().len() == 2 {
                    return Ok(());
                }
                Err("Both services not yet discovered")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap_or_else(|e| panic!("Failed to discover remote services:\n{}", e));

        T::sync(service_a1.service_hash().as_str().to_string(), TIMEOUT);
        T::sync(service_a2.service_hash().as_str().to_string(), TIMEOUT);

        // Listeners on host B
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b1 = node_b
            .service_builder(&service_name_1)
            .event()
            .open_or_create()
            .unwrap();
        let listener_b1 = service_b1.listener_builder().create().unwrap();

        let service_b2 = node_b
            .service_builder(&service_name_2)
            .event()
            .open_or_create()
            .unwrap();
        let listener_b2 = service_b2.listener_builder().create().unwrap();

        // === TEST ===
        // Distinct event ids sent on each service.
        let event_id_for_service_1 = EventId::new(101);
        let event_id_for_service_2 = EventId::new(202);

        notifier_a1
            .notify_with_custom_event_id(event_id_for_service_1)
            .unwrap();
        notifier_a2
            .notify_with_custom_event_id(event_id_for_service_2)
            .unwrap();

        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Each listener must receive the event id sent on the corresponding service.
        for (label, listener, expected_id) in [
            ("service_1", &listener_b1, event_id_for_service_1),
            ("service_2", &listener_b2, event_id_for_service_2),
        ] {
            T::retry(
                || match listener.try_wait_one().unwrap() {
                    Some(received_id) => {
                        if received_id != expected_id {
                            test_fail!("{}: received event id from a different service", label);
                        }
                        Ok(())
                    }
                    None => {
                        tunnel_a.propagate().unwrap();
                        tunnel_b.propagate().unwrap();
                        Err("not yet received")
                    }
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            )
            .unwrap_or_else(|e| panic!("{} failed: {}", label, e));
        }
    }
}
