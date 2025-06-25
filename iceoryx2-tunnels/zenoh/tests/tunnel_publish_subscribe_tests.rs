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
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::{assert_that, test_fail};
    use iceoryx2_tunnels_zenoh::*;

    fn mock_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_zenoh_tunnel_publish_subscribe_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
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
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

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
        let tunneled_services_a = tunnel_a.tunneled_services();
        assert_that!(tunneled_services_a.len(), eq 1);
        assert_that!(tunneled_services_a
            .contains(&String::from(iox_service_a.service_id().as_str())), eq true);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // Discover
        retry(
            || {
                tunnel_b.discover(Scope::Zenoh).unwrap();

                let tunneled_services = tunnel_b.tunneled_services();
                let success =
                    tunneled_services.contains(&String::from(iox_service_a.service_id().as_str()));

                if success {
                    return Ok(());
                }
                return Err("failed to discover remote service");
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
            tunnel_a.propagate();
            tunnel_b.propagate();

            // Receive
            retry(
                || {
                    match iox_subscriber_b.receive().unwrap() {
                        Some(iox_sample_received_b) => {
                            let iox_payload_received_b = iox_sample_received_b.payload();

                            // Check if we received the expected sample for this iteration
                            if *iox_payload_received_b == payload_data {
                                return Ok(());
                            } else {
                                return Err("received unexpected sample");
                            }
                        }
                        None => {
                            tunnel_a.propagate();
                            tunnel_b.propagate();
                            return Err("failed to receive expected sample");
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
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

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
        let tunneled_services_a = tunnel_a.tunneled_services();
        assert_that!(tunneled_services_a.len(), eq 1);
        assert_that!(tunneled_services_a
            .contains(&String::from(iox_service_a.service_id().as_str())), eq true);

        // [[ HOST B ]]
        // Tunnel
        let z_config_b = zenoh::Config::default();
        let iox_config_b = generate_isolated_config();
        let tunnel_config_b = TunnelConfig::default();
        let mut tunnel_b =
            Tunnel::<S>::create(&tunnel_config_b, &iox_config_b, &z_config_b).unwrap();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // Discover
        retry(
            || {
                tunnel_b.discover(Scope::Zenoh).unwrap();

                let tunneled_services = tunnel_b.tunneled_services();
                let success =
                    tunneled_services.contains(&String::from(iox_service_a.service_id().as_str()));

                if success {
                    return Ok(());
                }
                return Err("failed to discover remote service");
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
            tunnel_a.propagate();
            tunnel_b.propagate();

            // Receive
            retry(
                || {
                    match iox_subscriber_b.receive().unwrap() {
                        Some(iox_sample_received_b) => {
                            let iox_payload_received_b = iox_sample_received_b.payload();

                            // Check if we received the expected sample for this iteration
                            if *iox_payload_received_b == *payload_data.as_bytes() {
                                return Ok(());
                            } else {
                                return Err("received unexpected sample");
                            }
                        }
                        None => {
                            tunnel_a.propagate();
                            tunnel_b.propagate();
                            return Err("failed to receive expected sample");
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
        let tunneled_services_a = tunnel_a.tunneled_services();
        assert_that!(tunneled_services_a.len(), eq 1);
        assert_that!(tunneled_services_a
            .contains(&String::from(iox_service_a.service_id().as_str())), eq true);

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
        tunnel_a.propagate();

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
