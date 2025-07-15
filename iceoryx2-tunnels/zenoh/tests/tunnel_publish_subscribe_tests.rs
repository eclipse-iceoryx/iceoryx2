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
mod zenoh_tunnel_publish_subscribe {

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
            "test_zenoh_tunnel_publish_subscribe_{}",
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
        assert_that!(tunnel.active_channels().len(), eq 2);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Publisher(String::from(iox_service.service_id().as_str()))), eq true);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Subscriber(String::from(iox_service.service_id().as_str()))), eq true);
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
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Discover
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 2);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Publisher(String::from(iox_service.service_id().as_str()))), eq true);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Subscriber(String::from(iox_service.service_id().as_str()))), eq true);
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
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // ==================== TEST =====================

        // Discover
        tunnel.discover(Scope::Iceoryx).unwrap();
        assert_that!(tunnel.active_channels().len(), eq 2);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Publisher(String::from(iox_service.service_id().as_str()))), eq true);
        assert_that!(tunnel
            .active_channels()
            .contains(&ChannelInfo::Subscriber(String::from(iox_service.service_id().as_str()))), eq true);

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
                    assert_that!(iox_static_details.publish_subscribe(), eq iox_service.static_config());
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
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
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
            .contains(&ChannelInfo::Publisher(String::from(iox_service_b.service_id().as_str()))), eq true);
        assert_that!(tunnel_b
            .active_channels()
            .contains(&ChannelInfo::Subscriber(String::from(iox_service_b.service_id().as_str()))), eq true);

        // [[ HOST A ]]
        // Discover - announced service should be discovered via Zenoh
        retry(
            || {
                tunnel_a.discover(Scope::Zenoh).unwrap();

                let tunneled_ports = tunnel_a.active_channels();
                let tunneled_publisher = tunneled_ports.contains(&ChannelInfo::Publisher(
                    String::from(iox_service_b.service_id().as_str()),
                ));
                let tunneled_subscriber = tunneled_ports.contains(&ChannelInfo::Subscriber(
                    String::from(iox_service_b.service_id().as_str()),
                ));

                if tunneled_publisher && tunneled_subscriber {
                    return Ok(());
                }
                Err("failed to discover remote services")
            },
            TIME_BETWEEN_RETRIES,
            Some(MAX_RETRIES),
        );
    }

    fn propagates_n_struct_payloads<S: Service>(sample_count: usize) {
        const MAX_RETRIES: usize = 25;
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);

        #[derive(Debug, Clone, PartialEq, ZeroCopySend)]
        #[repr(C)]
        struct MyType {
            id: u32,
            value: f64,
            active: bool,
        }

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

        // Publisher
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .publish_subscribe::<MyType>()
            .open_or_create()
            .unwrap();
        let iox_publisher_a = iox_service_a.publisher_builder().create().unwrap();

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_ports_a = tunnel_a.active_channels();
        assert_that!(tunneled_ports_a.len(), eq 2);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Publisher(String::from(iox_service_a.service_id().as_str()))), eq true);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Subscriber(String::from(iox_service_a.service_id().as_str()))), eq true);

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
                let tunneled_publisher = tunneled_ports.contains(&ChannelInfo::Publisher(
                    String::from(iox_service_a.service_id().as_str()),
                ));
                let tunneled_subscriber = tunneled_ports.contains(&ChannelInfo::Subscriber(
                    String::from(iox_service_a.service_id().as_str()),
                ));

                if tunneled_publisher && tunneled_subscriber {
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

        // Subscriber
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .publish_subscribe::<MyType>()
            .open_or_create()
            .unwrap();
        let iox_subscriber_b = iox_service_b.subscriber_builder().create().unwrap();

        // ==================== TEST =====================

        for i in 0..sample_count {
            // Publish
            let payload_data = MyType {
                id: 42 + i as u32,
                value: 3.14 + i as f64,
                active: i % 2 == 0,
            };

            let iox_sample_sent_a = iox_publisher_a.loan_uninit().unwrap();
            let iox_sample_sent_a = iox_sample_sent_a.write_payload(payload_data.clone());
            iox_sample_sent_a.send().unwrap();

            // Propagate over tunnels
            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();

            // Receive
            retry(
                || {
                    match iox_subscriber_b.receive().unwrap() {
                        Some(iox_sample_received_b) => {
                            let iox_payload_received_b = iox_sample_received_b.payload();

                            // Check if we received the expected sample for this iteration
                            if *iox_payload_received_b == payload_data {
                                Ok(())
                            } else {
                                Err("received unexpected sample")
                            }
                        }
                        None => {
                            tunnel_a.propagate().unwrap();
                            tunnel_b.propagate().unwrap();
                            Err("failed to receive expected sample")
                        }
                    }
                },
                TIME_BETWEEN_RETRIES,
                Some(MAX_RETRIES),
            );
        }
    }

    #[test]
    fn propagates_one_struct_payload<S: Service>() {
        propagates_n_struct_payloads::<S>(1);
    }

    #[test]
    fn propagates_two_struct_payloads<S: Service>() {
        propagates_n_struct_payloads::<S>(2);
    }

    #[test]
    fn propagates_ten_struct_payloads<S: Service>() {
        propagates_n_struct_payloads::<S>(10);
    }

    fn propagates_n_slice_payloads<S: Service>(sample_count: usize) {
        const MAX_RETRIES: usize = 25;
        const TIME_BETWEEN_RETRIES: Duration = Duration::from_millis(250);
        const PAYLOAD_DATA_LENGTH: usize = 256;

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

        // Publisher
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();
        let iox_publisher_a = iox_service_a
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA_LENGTH)
            .create()
            .unwrap();

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_services_a = tunnel_a.active_channels();
        assert_that!(tunneled_services_a.len(), eq 2);
        assert_that!(tunneled_services_a
            .contains(&ChannelInfo::Publisher(String::from(iox_service_a.service_id().as_str()))), eq true);
        assert_that!(tunneled_services_a
            .contains(&ChannelInfo::Subscriber(String::from(iox_service_a.service_id().as_str()))), eq true);

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
                let tunneled_publisher = tunneled_ports.contains(&ChannelInfo::Publisher(
                    String::from(iox_service_a.service_id().as_str()),
                ));
                let tunneled_subscriber = tunneled_ports.contains(&ChannelInfo::Subscriber(
                    String::from(iox_service_a.service_id().as_str()),
                ));

                if tunneled_publisher && tunneled_subscriber {
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

        // Subscriber
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_b = iox_node_b
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();
        let iox_subscriber_b = iox_service_b.subscriber_builder().create().unwrap();

        // ==================== TEST =====================

        for i in 0..sample_count {
            // Publish
            let mut payload_data = String::with_capacity(PAYLOAD_DATA_LENGTH);
            for j in 0..PAYLOAD_DATA_LENGTH {
                let char_index = ((i * 7 + j * 13) % 26) as u8;
                let char_value = (b'A' + char_index) as char;
                payload_data.push(char_value);
            }

            let iox_sample_sent_a = iox_publisher_a
                .loan_slice_uninit(PAYLOAD_DATA_LENGTH)
                .unwrap();
            let iox_sample_sent_a = iox_sample_sent_a.write_from_slice(payload_data.as_bytes());
            iox_sample_sent_a.send().unwrap();

            // Propagate
            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();

            // Receive
            retry(
                || {
                    match iox_subscriber_b.receive().unwrap() {
                        Some(iox_sample_received_b) => {
                            let iox_payload_received_b = iox_sample_received_b.payload();

                            // Check if we received the expected sample for this iteration
                            if *iox_payload_received_b == *payload_data.as_bytes() {
                                Ok(())
                            } else {
                                Err("received unexpected sample")
                            }
                        }
                        None => {
                            tunnel_a.propagate().unwrap();
                            tunnel_b.propagate().unwrap();
                            Err("failed to receive expected sample")
                        }
                    }
                },
                TIME_BETWEEN_RETRIES,
                Some(MAX_RETRIES),
            );
        }
    }

    #[test]
    fn propagates_one_slice_payload<S: Service>() {
        propagates_n_slice_payloads::<S>(1);
    }

    #[test]
    fn propagates_two_slice_payloads<S: Service>() {
        propagates_n_slice_payloads::<S>(2);
    }

    #[test]
    fn propagates_ten_slice_payloads<S: Service>() {
        propagates_n_slice_payloads::<S>(10);
    }

    #[test]
    fn propagated_payloads_do_not_loop_back<S: Service>() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

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

        // Publisher
        let iox_node_a = NodeBuilder::new()
            .config(&iox_config_a)
            .create::<S>()
            .unwrap();
        let iox_service_a = iox_node_a
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();
        let iox_publisher_a = iox_service_a
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA.len())
            .create()
            .unwrap();

        // Subscriber
        let iox_subscriber_a = iox_service_a.subscriber_builder().create().unwrap();

        // Discover
        tunnel_a.discover(Scope::Iceoryx).unwrap();
        let tunneled_ports_a = tunnel_a.active_channels();
        assert_that!(tunneled_ports_a.len(), eq 2);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Publisher(String::from(iox_service_a.service_id().as_str()))), eq true);
        assert_that!(tunneled_ports_a
            .contains(&ChannelInfo::Subscriber(String::from(iox_service_a.service_id().as_str()))), eq true);

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Publish
        let iox_sample_a = iox_publisher_a
            .loan_slice_uninit(PAYLOAD_DATA.len())
            .unwrap();
        let iox_sample_a = iox_sample_a.write_from_slice(PAYLOAD_DATA.as_bytes());
        iox_sample_a.send().unwrap();

        // Receive - Sample should be received from local publisher
        while let Ok(Some(_)) = iox_subscriber_a.receive() {}

        // Propagate
        tunnel_a.propagate().unwrap();

        // Receive - Sample should not loop back and be received again
        if iox_subscriber_a.receive().unwrap().is_some() {
            test_fail!("sample looped back")
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
