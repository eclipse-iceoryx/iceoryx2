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

use iceoryx2_bb_conformance_test_macros::conformance_test_module;

#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod event_propagation {
    use core::fmt::Debug;
    use core::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2_bb_conformance_test_macros::conformance_test;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_tunnel::Tunnel;
    use iceoryx2_tunnel_backend::traits::{testing::Testing, Backend};

    fn generate_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "publish_subscribe_relay_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    fn propagate_events<S: Service, B: Backend<S> + Debug, T: Testing>(num: usize) {
        const TIMEOUT: Duration = Duration::from_millis(250);
        const MAX_ATTEMPTS: usize = 25;

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let backend_config_a = B::Config::default();
        let iceoryx_config_a = generate_isolated_config();
        let tunnel_config_a = iceoryx2_tunnel::Config::default();

        let mut tunnel_a =
            Tunnel::<S, B>::create(&tunnel_config_a, &iceoryx_config_a, &backend_config_a).unwrap();
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
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_id()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();
        let backend_config_b = B::Config::default();
        let tunnel_config_b = iceoryx2_tunnel::Config::default();
        let mut tunnel_b =
            Tunnel::<S, B>::create(&tunnel_config_b, &iceoryx_config_b, &backend_config_b).unwrap();

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
        );
        T::sync(service_a.service_id().as_str().to_string(), TIMEOUT);

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
                || match listener_b.try_wait_one().unwrap() {
                    Some(_event_id) => Ok(()),
                    None => {
                        tunnel_a.propagate().unwrap();
                        tunnel_b.propagate().unwrap();
                        Err("Failed to receive expected event")
                    }
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            );
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
        let tunnel_config_a = iceoryx2_tunnel::Config::default();
        let iceoryx_config_a = generate_isolated_config();
        let backend_config_a = B::Config::default();

        let mut tunnel_a =
            Tunnel::<S, B>::create(&tunnel_config_a, &iceoryx_config_a, &backend_config_a).unwrap();
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
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_id()), eq true);

        // --- Host B ---
        let tunnel_config_b = iceoryx2_tunnel::Config::default();
        let iceoryx_config_b = generate_isolated_config();
        let backend_config_b = B::Config::default();

        let mut tunnel_b =
            Tunnel::<S, B>::create(&tunnel_config_b, &iceoryx_config_b, &backend_config_b).unwrap();
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
        );

        T::sync(service_a.service_id().as_str().to_string(), TIMEOUT);

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
        listener_a.try_wait_all(|_| {}).unwrap();

        // Propagate over tunnels
        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Receive at listener b with retry
        T::retry(
            || match listener_b.try_wait_one().unwrap() {
                Some(_event_id) => Ok(()),
                None => {
                    tunnel_a.propagate().unwrap();
                    tunnel_b.propagate().unwrap();
                    Err("failed to receive expected event")
                }
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        );

        // Propagate a few times to see if there is a loop-back
        for _ in 0..5 {
            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();
            std::thread::sleep(Duration::from_millis(100));
        }

        // Notification should not have looped back from b to a
        let result = listener_a.try_wait_one();
        assert_that!(result, is_ok);
        let sample = result.unwrap();
        assert_that!(sample, is_none);
    }

    // TODO: Fix flaky
    #[conformance_test]
    pub fn multiple_events_are_consolidated_by_id<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let tunnel_config_a = iceoryx2_tunnel::Config::default();
        let iceoryx_config_a = generate_isolated_config();
        let backend_config_a = B::Config::default();

        let mut tunnel_a =
            Tunnel::<S, B>::create(&tunnel_config_a, &iceoryx_config_a, &backend_config_a).unwrap();
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
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_id()), eq true);

        // --- Host B ---
        let tunnel_config_b = iceoryx2_tunnel::Config::default();
        let iceoryx_config_b = generate_isolated_config();
        let backend_config_b = B::Config::default();

        let mut tunnel_b =
            Tunnel::<S, B>::create(&tunnel_config_b, &iceoryx_config_b, &backend_config_b).unwrap();
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
        );

        T::sync(service_a.service_id().as_str().to_string(), TIMEOUT);

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
                    .try_wait_all(|id| {
                        if id == event_a {
                            num_notifications_a += 1;
                        }
                        if id == event_b {
                            num_notifications_b += 1;
                        }
                        if id == event_c {
                            num_notifications_c += 1;
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
        );

        assert_that!(num_notifications_a, eq 1);
        assert_that!(num_notifications_b, eq 1);
        assert_that!(num_notifications_c, eq 1);
    }
}
