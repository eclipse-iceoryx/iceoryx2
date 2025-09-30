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

mod testing;

#[generic_tests::define]
mod zenoh_tunnel_events {

    use crate::testing::*;

    use std::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::service::static_config::StaticConfig;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::{assert_that, test_fail};
    use iceoryx2_services_discovery::service_discovery::Config as DiscoveryConfig;
    use iceoryx2_services_discovery::service_discovery::Service as DiscoveryService;
    use iceoryx2_tunnels_zenoh::*;

    use zenoh::Wait;

    fn mock_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_zenoh_tunnel_event_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn discovers_local_services_via_discovery_service<S: Service>() {
        // ==================== SETUP ====================

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();
        let iox_config = generate_isolated_config();

        // [[ DISCOVERY SERVICE ]]
        let discovery_config = DiscoveryConfig {
            publish_events: true,
            ..Default::default()
        };
        let mut discovery_service =
            DiscoveryService::<S>::create(&discovery_config, &iox_config).unwrap();

        // [[ HOST A ]]
        // Tunnel
        let z_config_a = zenoh::Config::default();
        let tunnel_config = TunnelConfig {
            discovery_service: Some("iox2://discovery/services/".into()),
        };

        let mut tunnel = Tunnel::<S>::create(&tunnel_config, &iox_config, &z_config_a).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 0);

        // Service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ DISCOVERY SERVICE ]]
        // Discover
        discovery_service.spin(|_| {}, |_| {}).unwrap();

        // [[ HOST A ]]
        // Respond to discovered services
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 2);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Notifier(String::from(iox_service.service_id().as_str()))), eq true);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Listener(String::from(iox_service.service_id().as_str()))), eq true);
    }

    #[test]
    fn discovers_local_services_via_tracker<S: Service>() {
        // ==================== SETUP ====================

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let z_config = zenoh::Config::default();
        let iox_config = generate_isolated_config();
        let tunnel_config = TunnelConfig::default();
        let mut tunnel = Tunnel::<S>::create(&tunnel_config, &iox_config, &z_config).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 0);

        // Service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Discover
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 2);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Notifier(String::from(iox_service.service_id().as_str()))), eq true);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Listener(String::from(iox_service.service_id().as_str()))), eq true);
    }

    #[test]
    fn announces_service_details_on_zenoh<S: Service>() {
        // ==================== SETUP ====================

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let iox_config = generate_isolated_config();
        let z_config = zenoh::Config::default();
        let tunnel_config = TunnelConfig::default();

        let mut tunnel = Tunnel::<S>::create(&tunnel_config, &iox_config, &z_config).unwrap();

        // Service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // Discover
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 2);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Notifier(String::from(iox_service.service_id().as_str()))), eq true);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Listener(String::from(iox_service.service_id().as_str()))), eq true);

        // Query Zenoh for Services
        let z_config_b = zenoh::config::Config::default();
        let z_session_b = zenoh::open(z_config_b.clone()).wait().unwrap();
        let z_reply_b = z_session_b
            .get(keys::service_details(iox_service.service_id()))
            .wait()
            .unwrap();
        match z_reply_b.recv_timeout(Duration::from_millis(100)) {
            Ok(Some(reply)) => match reply.result() {
                Ok(sample) => {
                    let iox_static_details: StaticConfig =
                        serde_json::from_slice(&sample.payload().to_bytes()).unwrap();
                    assert_that!(iox_static_details.service_id(), eq iox_service.service_id());
                    assert_that!(iox_static_details.name(), eq & iox_service_name);
                    assert_that!(iox_static_details.event(), eq iox_service.static_config());
                }
                Err(e) => test_fail!("error reading reply to type details query: {}", e),
            },
            Ok(None) => test_fail!("no reply to type details query"),
            Err(e) => test_fail!("error querying message type details from zenoh: {}", e),
        }
    }

    #[test]
    fn discovers_remote_services_via_zenoh<S: Service>() {
        const MAX_RETRIES: usize = 25;
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);

        // ==================== SETUP ====================

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let z_config_a = zenoh::Config::default();
        let iox_config_a = generate_isolated_config();
        let tunnel_config_a = TunnelConfig::default();
        let mut tunnel_a =
            Tunnel::<S>::create(&tunnel_config_a, &iox_config_a, &z_config_a).unwrap();
        assert_that!(tunnel_a.active_channels().len(), eq 0);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.active_channels().len(), eq 0);

        // Service
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Discover - nothing should be discovered
        tunnel_a.discover(Scope::Zenoh).unwrap();
        assert_that!(tunnel_a.active_channels().len(), eq 0);

        // [[ HOST B ]]
        // Discover - service should be announced
        tunnel_b.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel_b.active_channels().len(), eq 2);
        assert_that!(tunnel_b
            .active_channels()
            .contains(&ChannelInfo::Notifier(String::from(iox_service_b.service_id().as_str()))), eq true);
        assert_that!(tunnel_b
            .active_channels()
            .contains(&ChannelInfo::Listener(String::from(iox_service_b.service_id().as_str()))), eq true);

        // [[ HOST A ]]
        // Discover - announced service should be discovered via Zenoh
        retry(
            || {
                tunnel_a.discover(Scope::Zenoh).unwrap();

                let tunneled_ports = tunnel_a.active_channels();
                let tunneled_notifier = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_b.service_id().as_str()),
                ));
                let tunneled_listener = tunneled_ports.contains(&ChannelInfo::Listener(
                    String::from(iox_service_b.service_id().as_str()),
                ));

                if tunneled_notifier && tunneled_listener {
                    return Ok(());
                }
                Err("failed to discover remote services")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );
    }

    #[test]
    fn propagates_one_event<S: Service>() {
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const MAX_RETRIES: usize = 25;

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let z_config_a = zenoh::Config::default();
        let iox_config_a = generate_isolated_config();
        let tunnel_config_a = TunnelConfig::default();
        let mut tunnel_a =
            Tunnel::<S>::create(&tunnel_config_a, &iox_config_a, &z_config_a).unwrap();
        assert_that!(tunnel_a.active_channels().len(), eq 0);

        // Notifier
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();
        let iox_notifier_a = iox_service_a.notifier_builder().create().unwrap();

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_ports_a = tunnel_a.active_channels();
        assert_that!(tunneled_ports_a.len(), eq 2);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Notifier(String::from(iox_service_a.service_id().as_str()))), eq true);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Listener(String::from(iox_service_a.service_id().as_str()))), eq true);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.active_channels().len(), eq 0);

        // Discover
        retry(
            || {
                tunnel_b.discover(Scope::Zenoh).unwrap();

                let tunneled_ports = tunnel_b.active_channels();
                let tunneled_notifier = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_a.service_id().as_str()),
                ));
                let tunneled_listener = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_a.service_id().as_str()),
                ));

                if tunneled_notifier && tunneled_listener {
                    return Ok(());
                }
                Err("failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );

        // Wait for Zenoh's backgorund thread to establish match...
        let matched = wait_for_zenoh_match(
            keys::publish_subscribe(iox_service_a.service_id()),
            Duration::from_millis(1000),
        );
        assert_that!(matched, eq true);

        // Listener
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();
        let iox_listener_b = iox_service_b.listener_builder().create().unwrap();

        // ==================== TEST =====================
        // Send notification
        iox_notifier_a.notify().unwrap();

        // Propagate over tunnels
        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Receive with retry
        retry(
            || match iox_listener_b.try_wait_one().unwrap() {
                Some(_event_id) => Ok(()),
                None => {
                    tunnel_a.propagate().unwrap();
                    tunnel_b.propagate().unwrap();
                    Err("failed to receive expected event")
                }
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );
    }

    #[test]
    fn propagated_events_do_not_loop_back<S: Service>() {
        const MAX_RETRIES: usize = 25;
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let z_config_a = zenoh::Config::default();
        let iox_config_a = generate_isolated_config();
        let tunnel_config_a = TunnelConfig::default();
        let mut tunnel_a =
            Tunnel::<S>::create(&tunnel_config_a, &iox_config_a, &z_config_a).unwrap();
        assert_that!(tunnel_a.active_channels().len(), eq 0);

        // Notifier
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();
        let iox_notifier_a = iox_service_a.notifier_builder().create().unwrap();

        // Listener
        let iox_listener_a = iox_service_a.listener_builder().create().unwrap();

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_ports_a = tunnel_a.active_channels();
        assert_that!(tunneled_ports_a.len(), eq 2);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Notifier(String::from(iox_service_a.service_id().as_str()))), eq true);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Listener(String::from(iox_service_a.service_id().as_str()))), eq true);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.active_channels().len(), eq 0);

        // Discover
        retry(
            || {
                tunnel_b.discover(Scope::Zenoh).unwrap();

                let tunneled_ports = tunnel_b.active_channels();
                let tunneled_notifier = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_a.service_id().as_str()),
                ));
                let tunneled_listener = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_a.service_id().as_str()),
                ));

                if tunneled_notifier && tunneled_listener {
                    return Ok(());
                }

                Err("failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );

        // Wait for Zenoh's backgorund thread to establish match...
        let matched = wait_for_zenoh_match(
            keys::publish_subscribe(iox_service_a.service_id()),
            Duration::from_millis(1000),
        );
        assert_that!(matched, eq true);

        // Listener
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();
        let iox_listener_b = iox_service_b.listener_builder().create().unwrap();

        // ==================== TEST =====================

        // Send notification
        iox_notifier_a.notify().unwrap();

        // Drain the notification at host a
        iox_listener_a.try_wait_all(|_| {}).unwrap();

        // Propagate over tunnels
        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Receive at listener b with retry
        retry(
            || match iox_listener_b.try_wait_one().unwrap() {
                Some(_event_id) => Ok(()),
                None => {
                    tunnel_a.propagate().unwrap();
                    tunnel_b.propagate().unwrap();
                    Err("failed to receive expected event")
                }
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );

        // Propagate a few times to see if there is a loop-back
        for _ in 0..5 {
            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();
            std::thread::sleep(Duration::from_millis(100));
        }

        // Notification should not have looped back from b to a
        let result = iox_listener_a.try_wait_one();
        assert_that!(result, is_ok);
        let sample = result.unwrap();
        assert_that!(sample, is_none);
    }

    #[test]
    fn multiple_events_are_consolidated_by_id<S: Service>() {
        const MAX_RETRIES: usize = 25;
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let z_config_a = zenoh::Config::default();
        let iox_config_a = generate_isolated_config();
        let tunnel_config_a = TunnelConfig::default();
        let mut tunnel_a =
            Tunnel::<S>::create(&tunnel_config_a, &iox_config_a, &z_config_a).unwrap();
        assert_that!(tunnel_a.active_channels().len(), eq 0);

        // Notifier
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();
        let iox_notifier_a = iox_service_a.notifier_builder().create().unwrap();

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_ports_a = tunnel_a.active_channels();
        assert_that!(tunneled_ports_a.len(), eq 2);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Notifier(String::from(iox_service_a.service_id().as_str()))), eq true);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Listener(String::from(iox_service_a.service_id().as_str()))), eq true);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.active_channels().len(), eq 0);

        // Discover
        retry(
            || {
                tunnel_b.discover(Scope::Zenoh).unwrap();

                let tunneled_ports = tunnel_b.active_channels();
                let tunneled_notifier = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_a.service_id().as_str()),
                ));
                let tunneled_listener = tunneled_ports.contains(&ChannelInfo::Notifier(
                    String::from(iox_service_a.service_id().as_str()),
                ));

                if tunneled_notifier && tunneled_listener {
                    return Ok(());
                }
                Err("failed to discover remote service")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );

        // Wait for Zenoh's backgorund thread to establish match...
        let matched = wait_for_zenoh_match(
            keys::publish_subscribe(iox_service_a.service_id()),
            Duration::from_millis(1000),
        );
        assert_that!(matched, eq true);

        // Listener
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .event()
            .open_or_create()
            .unwrap();
        let iox_listener_b = iox_service_b.listener_builder().create().unwrap();

        // ==================== TEST =====================
        // Send multiple notifications on different event ids
        let event_a = EventId::new(42);
        let event_b = EventId::new(73);
        let event_c = EventId::new(127);

        const NUM_NOTIFICATIONS: usize = 10;
        for _ in 0..NUM_NOTIFICATIONS {
            iox_notifier_a.notify_with_custom_event_id(event_a).unwrap();
            iox_notifier_a.notify_with_custom_event_id(event_b).unwrap();
            iox_notifier_a.notify_with_custom_event_id(event_c).unwrap();
        }

        // Propagate over tunnels
        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Receive with retry
        let mut num_notifications_a = 0;
        let mut num_notifications_b = 0;
        let mut num_notifications_c = 0;

        retry(
            || {
                iox_listener_b
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
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );

        assert_that!(num_notifications_a, eq 1);
        assert_that!(num_notifications_b, eq 1);
        assert_that!(num_notifications_c, eq 1);
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
