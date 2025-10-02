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

#[generic_tests::define]
mod tunnel_publish_subscribe_tests {
    use core::fmt::Debug;
    use core::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2::service::Service;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_tunnel_core::Tunnel;
    use iceoryx2_tunnel_traits::{testing::Testing, Transport};

    fn generate_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_tunnel_publish_subscribe_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    fn propagate_struct_payloads<S: Service, T: Transport, U: Testing>(num: usize) {
        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);

        #[derive(Debug, Clone, PartialEq, ZeroCopySend)]
        #[repr(C)]
        struct MyType {
            id: u32,
            value: f64,
            active: bool,
        }

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let iceoryx_config_a = generate_isolated_config();
        let transport_config_a = T::Config::default();
        let tunnel_config_a = iceoryx2_tunnel_core::Config::default();
        let mut tunnel_a =
            Tunnel::<S, T>::create(&tunnel_config_a, &iceoryx_config_a, &transport_config_a)
                .unwrap();

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .publish_subscribe::<MyType>()
            .open_or_create()
            .unwrap();
        let publisher_a = service_a.publisher_builder().create().unwrap();

        tunnel_a.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_id()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();
        let transport_config_b = T::Config::default();
        let tunnel_config_b = iceoryx2_tunnel_core::Config::default();
        let mut tunnel_b =
            Tunnel::<S, T>::create(&tunnel_config_b, &iceoryx_config_b, &transport_config_b)
                .unwrap();

        // Wait for tunnel on host b to discover the service on host A
        U::retry(
            || {
                tunnel_b.discover_over_transport().unwrap();
                let service_discovered = tunnel_b.tunneled_services().len() == 1;
                if service_discovered {
                    return Ok(());
                }
                Err("Failed to discover remote services")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        );

        // Wait for zenoh to wake up and process the discovered service
        U::sync(service_a.service_id().as_str().to_string(), TIMEOUT);

        // Create a subscribe to connect to the tunneled service
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .publish_subscribe::<MyType>()
            .open_or_create()
            .unwrap();
        let subscriber_b = service_b.subscriber_builder().create().unwrap();

        // === TEST ===
        for i in 0..num {
            // Publish
            let payload_data = MyType {
                id: 42 + i as u32,
                value: 3.14 + i as f64,
                active: i % 2 == 0,
            };

            let sample_sent_at_a = publisher_a.loan_uninit().unwrap();
            let sample_sent_at_a = sample_sent_at_a.write_payload(payload_data.clone());
            sample_sent_at_a.send().unwrap();

            // Propagate over tunnels
            U::retry(
                || {
                    match subscriber_b.receive().unwrap() {
                        Some(sample_received_at_b) => {
                            let payload_received_at_b = sample_received_at_b.payload();

                            // Check if we received the expected sample for this iteration
                            if *payload_received_at_b == payload_data {
                                Ok(())
                            } else {
                                Err("received unexpected sample")
                            }
                        }
                        None => {
                            tunnel_a.relay().unwrap();
                            tunnel_b.relay().unwrap();
                            Err("failed to receive expected sample")
                        }
                    }
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            );
        }
    }

    fn propagate_slice_payloads<S: Service, T: Transport, U: Testing>(num: usize) {
        set_log_level(LogLevel::Debug);

        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);
        const PAYLOAD_DATA_LENGTH: usize = 256;

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
        let iceoryx_config_a = generate_isolated_config();
        let transport_config_a = T::Config::default();
        let tunnel_config_a = iceoryx2_tunnel_core::Config::default();
        let mut tunnel_a =
            Tunnel::<S, T>::create(&tunnel_config_a, &iceoryx_config_a, &transport_config_a)
                .unwrap();

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();
        let publisher_a = service_a
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA_LENGTH)
            .create()
            .unwrap();

        tunnel_a.discover_over_iceoryx().unwrap();
        assert_that!(tunnel_a.tunneled_services().len(), eq 1);
        assert_that!(tunnel_a.tunneled_services().contains(service_a.service_id()), eq true);

        // --- Host B ---
        let iceoryx_config_b = generate_isolated_config();
        let transport_config_b = T::Config::default();
        let tunnel_config_b = iceoryx2_tunnel_core::Config::default();
        let mut tunnel_b =
            Tunnel::<S, T>::create(&tunnel_config_b, &iceoryx_config_b, &transport_config_b)
                .unwrap();

        // Wait for tunnel on host b to discover the service on host A
        U::retry(
            || {
                tunnel_b.discover_over_transport().unwrap();
                let service_discovered = tunnel_b.tunneled_services().len() == 1;
                if service_discovered {
                    return Ok(());
                }
                Err("Failed to discover remote services")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        );

        // Wait for zenoh to wake up and process the discovered service
        U::sync(service_a.service_id().as_str().to_string(), TIMEOUT);

        // Create a subscribe to connect to the tunneled service
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();
        let subscriber_b = service_b.subscriber_builder().create().unwrap();

        // === TEST ===
        for i in 0..num {
            // Publish
            let mut payload_data = String::with_capacity(PAYLOAD_DATA_LENGTH);
            for j in 0..PAYLOAD_DATA_LENGTH {
                let char_index = ((i * 7 + j * 13) % 26) as u8;
                let char_value = (b'A' + char_index) as char;
                payload_data.push(char_value);
            }

            let sample_sent_at_a = publisher_a.loan_slice_uninit(PAYLOAD_DATA_LENGTH).unwrap();
            let sample_sent_at_a = sample_sent_at_a.write_from_slice(payload_data.as_bytes());
            sample_sent_at_a.send().unwrap();

            // Propagate over tunnels
            U::retry(
                || {
                    match subscriber_b.receive().unwrap() {
                        Some(sample_received_at_b) => {
                            let payload_received_at_b = sample_received_at_b.payload();

                            // Check if we received the expected sample for this iteration
                            if *payload_received_at_b == *payload_data.as_bytes() {
                                Ok(())
                            } else {
                                Err("received unexpected sample")
                            }
                        }
                        None => {
                            tunnel_a.relay().unwrap();
                            tunnel_b.relay().unwrap();
                            Err("failed to receive expected sample")
                        }
                    }
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            );
        }
    }

    #[test]
    fn propagates_struct_payload<S: Service, T: Transport, U: Testing>() {
        propagate_struct_payloads::<S, T, U>(1);
    }

    #[test]
    fn propagates_struct_payload_many<S: Service, T: Transport, U: Testing>() {
        propagate_struct_payloads::<S, T, U>(10);
    }

    #[test]
    fn propagates_slice_payload<S: Service, T: Transport, U: Testing>() {
        propagate_slice_payloads::<S, T, U>(1);
    }

    #[test]
    fn propagates_slice_payload_many<S: Service, T: Transport, U: Testing>() {
        propagate_slice_payloads::<S, T, U>(10);
    }

    #[test]
    fn propagated_payloads_do_not_loop_back<S: Service, T: Transport, U: Testing>() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

        // === SETUP ===
        let service_name = generate_service_name();

        let transport_config = T::Config::default();
        let iceoryx_config = generate_isolated_config();
        let tunnel_config = iceoryx2_tunnel_core::Config::default();
        let mut tunnel =
            Tunnel::<S, T>::create(&tunnel_config, &iceoryx_config, &transport_config).unwrap();

        // Publisher
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<S>()
            .unwrap();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();
        let publisher = service
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA.len())
            .create()
            .unwrap();

        // Subscriber
        let subscriber = service.subscriber_builder().create().unwrap();

        // Discover
        tunnel.discover_over_iceoryx().unwrap();
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel.tunneled_services().contains(service.service_id()), eq true);

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Publish
        let sample = publisher.loan_slice_uninit(PAYLOAD_DATA.len()).unwrap();
        let sample = sample.write_from_slice(PAYLOAD_DATA.as_bytes());
        sample.send().unwrap();

        // Receive - Sample should be received from local publisher
        while let Ok(Some(_)) = subscriber.receive() {}

        // Propagate
        tunnel.relay().unwrap();

        // Receive - Sample should not loop back and be received again
        if subscriber.receive().unwrap().is_some() {
            test_fail!("sample looped back")
        }
    }

    #[cfg(feature = "tunnel_zenoh")]
    #[instantiate_tests(<iceoryx2::service::ipc::Service, iceoryx2_tunnel_zenoh::Transport, iceoryx2_tunnel_zenoh::testing::Testing>)]
    mod ipc_zenoh {}
    #[cfg(feature = "tunnel_zenoh")]
    #[instantiate_tests(<iceoryx2::service::local::Service, iceoryx2_tunnel_zenoh::Transport, iceoryx2_tunnel_zenoh::testing::Testing>)]
    mod local_zenoh {}
}
