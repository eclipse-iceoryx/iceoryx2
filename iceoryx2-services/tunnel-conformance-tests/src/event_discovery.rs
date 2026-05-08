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
pub mod event_discovery {

    use alloc::format;
    use core::fmt::Debug;
    use core::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_services_common::DiscoveryEvent;
    use iceoryx2_services_discovery::service_discovery::Tracker;
    use iceoryx2_services_tunnel::Config as TunnelConfig;
    use iceoryx2_services_tunnel::Tunnel;
    use iceoryx2_services_tunnel_backend::traits::Backend;
    use iceoryx2_services_tunnel_backend::traits::testing::Testing;

    // TODO: Move to iceoryx2::testing
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;

    const DISCOVERY_TOPIC: &str = "iox2://discovery/services/";

    fn generate_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "event_discovery_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[conformance_test]
    pub fn discovers_added_and_removed_services_via_subscriber<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        // === SETUP ===
        let iceoryx_config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<S>()
            .unwrap();
        let service = node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service.service_hash();

        // Set up discovery service manually to better simulate test scenario.
        let discovery_service_name: ServiceName = DISCOVERY_TOPIC.try_into().unwrap();
        let discovery_service = node
            .service_builder(&discovery_service_name)
            .publish_subscribe::<DiscoveryEvent>()
            .max_publishers(1)
            .open_or_create()
            .unwrap();
        let discovery_publisher = discovery_service.publisher_builder().create().unwrap();

        // Capture the full StaticConfig for the user-created service so we can inject Added
        // into the discovery service manually.
        let mut tracker = Tracker::<S>::new(&iceoryx_config);
        tracker.sync().unwrap();
        let static_config = tracker.get(&service_hash).unwrap().static_details.clone();

        // Create the tunnel.
        let tunnel_config = TunnelConfig {
            discovery_service: Some(DISCOVERY_TOPIC.into()),
        };
        let mut tunnel =
            Tunnel::<S, B>::create(&tunnel_config, &iceoryx_config, &B::Config::default()).unwrap();

        // === ADDITION ===
        discovery_publisher
            .send_copy(DiscoveryEvent::Added(static_config))
            .unwrap();

        tunnel.discover_over_iceoryx().unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel.tunneled_services().contains(&service_hash), eq true);

        // === REMOVAL ===
        discovery_publisher
            .send_copy(DiscoveryEvent::Removed(service_hash))
            .unwrap();

        tunnel.discover_over_iceoryx().unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 0);
        assert_that!(tunnel.tunneled_services().contains(&service_hash), eq false);
    }

    #[conformance_test]
    pub fn discovers_added_and_removed_services_via_tracker<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        // === SETUP ===
        let iceoryx_config = generate_isolated_config();
        let service_name = generate_service_name();
        let tunnel_config = TunnelConfig::default();
        let mut tunnel =
            Tunnel::<S, B>::create(&tunnel_config, &iceoryx_config, &B::Config::default()).unwrap();

        // === ADDITION ===
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<S>()
            .unwrap();
        let service = node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service.service_hash();

        tunnel.discover_over_iceoryx().unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel.tunneled_services().contains(&service_hash), eq true);

        // === REMOVAL ===
        drop(service);

        tunnel.discover_over_iceoryx().unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 0);
        assert_that!(tunnel.tunneled_services().contains(&service_hash), eq false);
    }

    #[conformance_test]
    pub fn discovers_added_and_removed_services_via_backend<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 5;

        // === SETUP ===
        let service_name = generate_service_name();

        // Host A
        let iceoryx_config_a = generate_isolated_config();
        let backend_config_a = B::Config::default();
        let tunnel_config_a = TunnelConfig::default();
        let mut tunnel_a =
            Tunnel::<S, B>::create(&tunnel_config_a, &iceoryx_config_a, &backend_config_a).unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        // Host B
        let iceoryx_config_b = generate_isolated_config();
        let backend_config_b = B::Config::default();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S, B>::create(&tunnel_config_b, &iceoryx_config_b, &backend_config_b).unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // === ADDITION ===
        // Create a service on Host B
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_b.service_hash();

        // Initially, Host A discovers no remote services.
        tunnel_a.discover_over_backend().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        // Host B discovers the local service.
        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 1);
        assert_that!(tunnel_b.tunneled_services().contains(&service_hash), eq true);

        // Host B propagates the service over the Backend.
        T::retry(
            || {
                // The service becomes visible to Host A.
                tunnel_a.discover_over_backend().unwrap();

                if tunnel_a.tunneled_services().len() == 1 {
                    return Ok(());
                }
                Err("Failed to discover remote services")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
        assert_that!(tunnel_a.tunneled_services().contains(&service_hash), eq true);

        // === REMOVAL ===
        // Remove the service on Host B
        drop(service_b);
        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // Host B propagates the service over the Backend.
        T::retry(
            || {
                // The service is dropped on Host A.
                tunnel_a.discover_over_backend().unwrap();

                if tunnel_a.tunneled_services().is_empty() {
                    return Ok(());
                }
                Err("Failed to detect remote service removal")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        assert_that!(tunnel_a.tunneled_services().len(), eq 0);
        assert_that!(tunnel_a.tunneled_services().contains(&service_hash), eq false);
    }

    #[conformance_test]
    pub fn aggregates_announcements_from_multiple_hosts<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 5;

        // === SETUP ===
        let service_name = generate_service_name();

        // Host A — observer, announces nothing locally.
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_a,
            &B::Config::default(),
        )
        .unwrap();

        // Host B — announces the service.
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_b,
            &B::Config::default(),
        )
        .unwrap();

        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_b.service_hash();

        // Host C — announces the same service (same name → same hash).
        let iceoryx_config_c = generate_isolated_config();
        let mut tunnel_c = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_c,
            &B::Config::default(),
        )
        .unwrap();

        let node_c = NodeBuilder::new()
            .config(&iceoryx_config_c)
            .create::<S>()
            .unwrap();
        let service_c = node_c
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();

        // === ANNOUNCE FROM B AND C ===
        tunnel_b.discover_over_iceoryx().unwrap();
        tunnel_c.discover_over_iceoryx().unwrap();

        // Host A discovers the service exactly once despite two hosts announcing.
        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 1);

        // === REMOVE FROM B ===
        drop(service_b);
        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // Host A keeps the service since one remote still offering service
        for _ in 0..MAX_RETRIES {
            tunnel_a.discover_over_backend().unwrap();
        }
        assert_that!(tunnel_a.tunneled_services().contains(&service_hash), eq true);

        // === REMOVE FROM C ===
        drop(service_c);
        tunnel_c.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_c.tunneled_services().len(), eq 0);

        // Host A observes the service removal.
        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().is_empty() {
                    return Ok(());
                }
                Err("Failed to detect remote service removal")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();
    }

    #[conformance_test]
    pub fn detects_ungraceful_remote_departure<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(500);
        const MAX_RETRIES: usize = 20;

        // === SETUP ===
        let service_name = generate_service_name();

        // Host A
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_a,
            &B::Config::default(),
        )
        .unwrap();

        // Host B
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_b,
            &B::Config::default(),
        )
        .unwrap();
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_b.service_hash();

        // === ADDITION ===
        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 1);

        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        // === UNGRACEFUL DEPARTURE ===
        drop(service_b);
        drop(node_b);
        drop(tunnel_b);

        T::retry(
            || {
                // Service from crashed tunnel should be detected as removed.
                tunnel_a.discover_over_backend().unwrap();

                if tunnel_a.tunneled_services().is_empty() {
                    return Ok(());
                }
                Err("Failed to detect ungraceful remote departure")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();
    }

    #[conformance_test]
    pub fn discovers_pre_existing_remote_services<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 5;

        // === SETUP — Host B announces FIRST ===
        let service_name = generate_service_name();

        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_b,
            &B::Config::default(),
        )
        .unwrap();

        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_b.service_hash();

        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 1);

        // === LATE-JOINING HOST A ===
        // Host A is created after Host B has already announced.
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_a,
            &B::Config::default(),
        )
        .unwrap();

        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to discover pre-existing remote service via history replay")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
    }

    #[conformance_test]
    pub fn rediscovers_service_after_removal<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 5;

        // === SETUP ===
        let service_name = generate_service_name();

        // Host A — observer.
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_a,
            &B::Config::default(),
        )
        .unwrap();

        // Host B — announces, removes, re-announces the same service.
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_b,
            &B::Config::default(),
        )
        .unwrap();
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();

        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_b.service_hash();

        // === FIRST ADDITION ===
        tunnel_b.discover_over_iceoryx().unwrap();

        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        // === REMOVAL ===
        drop(service_b);
        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().is_empty() {
                    return Ok(());
                }
                Err("Failed to detect remote service removal")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        // === SECOND ADDITION ===
        // Recreate the service with the same name (same hash).
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        assert_that!(*service_b.service_hash(), eq service_hash);

        tunnel_b.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_b.tunneled_services().contains(&service_hash), eq true);

        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to rediscover service after removal")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();
    }

    #[conformance_test]
    pub fn ignores_duplicate_added_events<S: Service, B: Backend<S> + Debug, T: Testing>() {
        // === SETUP ===
        let iceoryx_config = generate_isolated_config();
        let service_name = generate_service_name();
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<S>()
            .unwrap();
        let service = node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service.service_hash();

        // Set up discovery service manually so we can inject events.
        let discovery_service_name: ServiceName = DISCOVERY_TOPIC.try_into().unwrap();
        let discovery_service = node
            .service_builder(&discovery_service_name)
            .publish_subscribe::<DiscoveryEvent>()
            .max_publishers(1)
            .open_or_create()
            .unwrap();
        let discovery_publisher = discovery_service.publisher_builder().create().unwrap();

        let mut tracker = Tracker::<S>::new(&iceoryx_config);
        tracker.sync().unwrap();
        let static_config = tracker.get(&service_hash).unwrap().static_details.clone();

        let tunnel_config = TunnelConfig {
            discovery_service: Some(DISCOVERY_TOPIC.into()),
        };
        let mut tunnel =
            Tunnel::<S, B>::create(&tunnel_config, &iceoryx_config, &B::Config::default()).unwrap();

        // === DUPLICATE ANNOUNCEMENT ===
        // Inject the same Added event twice. The second one must be a no-op.
        discovery_publisher
            .send_copy(DiscoveryEvent::Added(static_config.clone()))
            .unwrap();
        discovery_publisher
            .send_copy(DiscoveryEvent::Added(static_config))
            .unwrap();

        tunnel.discover_over_iceoryx().unwrap();

        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel.tunneled_services().contains(&service_hash), eq true);
    }

    #[conformance_test]
    pub fn lifecycle_with_local_service_on_one_host<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 5;

        // === SETUP ===
        let service_name = generate_service_name();

        // Host A — no local user; mirrors Host B's service in via the backend.
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_a,
            &B::Config::default(),
        )
        .unwrap();

        // Host B — will own the service.
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_b,
            &B::Config::default(),
        )
        .unwrap();

        // === ADDITION ===
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_b.service_hash();

        tunnel_b.discover_over_iceoryx().unwrap();

        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        // Local discovery on Host A must not tear down the mirror.
        for _ in 0..5 {
            tunnel_a.discover_over_iceoryx().unwrap();
            assert_that!(tunnel_a.tunneled_services().contains(&service_hash), eq true);
        }

        // === REMOVAL ===
        drop(service_b);
        drop(node_b);

        // Host B detects the local user is gone and tears down its port.
        T::retry(
            || {
                tunnel_b.discover_over_iceoryx().unwrap();
                if tunnel_b.tunneled_services().is_empty() {
                    return Ok(());
                }
                Err("Failed to detect local service removal")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        // Host A receives the removal over the backend and tears down the mirror.
        T::retry(
            || {
                tunnel_a.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().is_empty() {
                    return Ok(());
                }
                Err("Failed to tear down mirrored service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();
    }

    #[conformance_test]
    pub fn lifecycle_with_local_service_on_two_hosts<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 5;

        // === SETUP ===
        let service_name = generate_service_name();

        // Host A — will own the service first.
        let iceoryx_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_a,
            &B::Config::default(),
        )
        .unwrap();

        // Host B — will mirror Host A's service before a local
        // offerer appears
        let iceoryx_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S, B>::create(
            &TunnelConfig::default(),
            &iceoryx_config_b,
            &B::Config::default(),
        )
        .unwrap();

        // === ADDITION ===
        // Host A has a local
        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        let service_hash = *service_a.service_hash();

        tunnel_a.discover_over_iceoryx().unwrap();

        // Wait until Host B mirrors Host A's service via the backend before
        // adding B's own local user.
        T::retry(
            || {
                tunnel_b.discover_over_backend().unwrap();
                if tunnel_b.tunneled_services().contains(&service_hash) {
                    return Ok(());
                }
                Err("Failed to mirror remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();

        // Host B has a local offerer for the same service
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        assert_that!(*service_b.service_hash(), eq service_hash);

        tunnel_b.discover_over_iceoryx().unwrap();

        // === REMOVAL ===
        // Local offerer in host B is removed
        drop(service_b);
        drop(node_b);

        // Host A is still offering. Host B must retain the mirror across
        // repeated discovery cycles.
        for _ in 0..5 {
            tunnel_b.discover_over_iceoryx().unwrap();
            tunnel_b.discover_over_backend().unwrap();
            assert_that!(tunnel_b.tunneled_services().contains(&service_hash), eq true);
        }

        // Local offerer in host A is removed
        drop(service_a);
        drop(node_a);

        // No offerer remains anywhere. Both hosts tear down.
        T::retry(
            || {
                tunnel_a.discover_over_iceoryx().unwrap();
                tunnel_a.discover_over_backend().unwrap();
                tunnel_b.discover_over_iceoryx().unwrap();
                tunnel_b.discover_over_backend().unwrap();
                if tunnel_a.tunneled_services().is_empty()
                    && tunnel_b.tunneled_services().is_empty()
                {
                    return Ok(());
                }
                Err("Failed to tear down service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        )
        .unwrap();
    }
}
