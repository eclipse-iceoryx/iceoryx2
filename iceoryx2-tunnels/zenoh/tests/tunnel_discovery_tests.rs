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
mod zenoh_tunnel_discovery {

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
            "test_zenoh_tunnel_discovery_{}",
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
        assert_that!(tunnel.tunneled_services().len(), eq 0);

        // Service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ DISCOVERY SERVICE ]]
        // Discover
        discovery_service.spin(|_| {}, |_| {}).unwrap();

        // [[ HOST A ]]
        // Respond to discovered services
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel
            .tunneled_services()
            .contains(&String::from(iox_service.service_id().as_str())), eq true);
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
        assert_that!(tunnel.tunneled_services().len(), eq 0);

        // Service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Discover
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel
            .tunneled_services()
            .contains(&String::from(iox_service.service_id().as_str())), eq true);
    }

    #[test]
    fn announces_service_details_on_zenoh<S: Service>() {
        // ==================== SETUP ====================

        // [[ COMMON ]]
        let iox_service_name = mock_service_name();

        // [[ HOST A ]]
        // Tunnel
        let iox_config_a = generate_isolated_config();
        let z_config_a = zenoh::Config::default();
        let tunnel_config_a = TunnelConfig::default();

        let mut tunnel_a =
            Tunnel::<S>::create(&tunnel_config_a, &iox_config_a, &z_config_a).unwrap();

        // Service
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_services_a = tunnel_a.tunneled_services();
        assert_that!(tunneled_services_a.len(), eq 1);
        assert_that!(tunneled_services_a
            .contains(&String::from(iox_service_a.service_id().as_str())), eq true);

        // Query Zenoh for Services
        let z_config_b = zenoh::config::Config::default();
        let z_session_b = zenoh::open(z_config_b.clone()).wait().unwrap();
        let z_reply_b = z_session_b
            .get(keys::service_details(iox_service_a.service_id()))
            .wait()
            .unwrap();
        match z_reply_b.recv_timeout(Duration::from_millis(100)) {
            Ok(Some(reply)) => match reply.result() {
                Ok(sample) => {
                    let iox_static_details: StaticConfig =
                        serde_json::from_slice(&sample.payload().to_bytes()).unwrap();
                    assert_that!(iox_static_details.service_id(), eq iox_service_a.service_id());
                    assert_that!(iox_static_details.name(), eq & iox_service_name);
                    assert_that!(iox_static_details.publish_subscribe(), eq iox_service_a.static_config());
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
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // Service
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Discover - nothing should be discovered
        tunnel_a.discover(Scope::Zenoh).unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        // [[ HOST B ]]
        // Discover - service should be announced
        tunnel_b.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 1);
        assert_that!(tunnel_b
            .tunneled_services()
            .contains(&String::from(iox_service_b.service_id().as_str())), eq true);

        // [[ HOST A ]]
        // Discover - announced service should be discovered via Zenoh
        retry(
            || {
                tunnel_a.discover(Scope::Zenoh).unwrap();

                let tunneled_services = tunnel_a.tunneled_services();
                let success =
                    tunneled_services.contains(&String::from(iox_service_b.service_id().as_str()));

                if success {
                    return Ok(());
                }
                return Err("failed to discover remote service");
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
