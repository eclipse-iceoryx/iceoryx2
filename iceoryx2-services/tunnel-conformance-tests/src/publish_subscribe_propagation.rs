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
pub mod publish_subscribe_propagation {
    use alloc::string::{String, ToString};
    use core::fmt::Debug;
    use core::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2::service::Service;
    use iceoryx2::testing::generate_service_name;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_services_tunnel::Tunnel;
    use iceoryx2_services_tunnel_backend::traits::{Backend, testing::Testing};

    #[derive(Default, Debug, Clone, PartialEq, ZeroCopySend)]
    #[repr(C)]
    pub struct MyHeader {
        pub version: i32,
        pub timestamp: u64,
    }

    #[derive(Debug, Clone, PartialEq, ZeroCopySend)]
    #[repr(C)]
    struct MyType {
        id: u32,
        value: f64,
        active: bool,
    }

    fn propagate_struct_payloads<S: Service, B: Backend<S> + Debug, T: Testing>(num: usize) {
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

        let node_a = NodeBuilder::new()
            .config(&iceoryx_config_a)
            .create::<S>()
            .unwrap();
        let service_a = node_a
            .service_builder(&service_name)
            .publish_subscribe::<MyType>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let publisher_a = service_a.publisher_builder().create().unwrap();

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

        // Wait for tunnel on host B to discover the service on host A
        T::retry(
            || {
                tunnel_b.discover_over_backend().unwrap();
                let service_discovered = tunnel_b.tunneled_services().len() == 1;
                if service_discovered {
                    return Ok(());
                }
                Err("No services discovered")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap_or_else(|e| panic!("Failed to discover remote services:\n{}", e));

        T::sync(service_a.service_hash().as_str().to_string(), TIMEOUT);

        // Create a subscriber to connect to the tunneled service
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .publish_subscribe::<MyType>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let subscriber_b = service_b.subscriber_builder().create().unwrap();

        // === TEST ===
        for i in 0..num {
            // Publish
            let user_header_sent_at_a = MyHeader {
                version: 0,
                timestamp: 1000000000 + i as u64,
            };
            let payload_sent_at_a = MyType {
                id: 42 + i as u32,
                value: core::f64::consts::PI + i as f64,
                active: i % 2 == 0,
            };

            let mut sample_sent_at_a = publisher_a.loan_uninit().unwrap();
            *sample_sent_at_a.user_header_mut() = user_header_sent_at_a.clone();
            let sample_sent_at_a = sample_sent_at_a.write_payload(payload_sent_at_a.clone());
            sample_sent_at_a.send().unwrap();

            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();

            // Sync backends will see the sample on the first attempt; async
            // backends drive delivery forward via the in-closure propagate.
            T::retry(
                || match subscriber_b.receive().unwrap() {
                    Some(sample_received_at_b) => {
                        let user_header_received_at_b = sample_received_at_b.user_header();
                        let payload_received_at_b = sample_received_at_b.payload();

                        if *user_header_received_at_b != user_header_sent_at_a {
                            return Err("Failed to receive user header");
                        }
                        if *payload_received_at_b != payload_sent_at_a {
                            return Err("Failed to receive payload");
                        };

                        Ok(())
                    }
                    None => {
                        tunnel_a.propagate().unwrap();
                        tunnel_b.propagate().unwrap();
                        Err("Failed to receive sample")
                    }
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            )
            .unwrap_or_else(|e| panic!("Failed to propagate over tunnel:\n{}", e));
        }
    }

    fn propagate_slice_payloads<S: Service, B: Backend<S> + Debug, T: Testing>(num: usize) {
        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);
        const PAYLOAD_DATA_LENGTH: usize = 256;

        // === SETUP ===
        let service_name = generate_service_name();

        // --- Host A ---
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
        let service_a = node_a
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let publisher_a = service_a
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA_LENGTH)
            .create()
            .unwrap();

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
                Err("No services discovered")
            },
            TIMEOUT,
            Some(MAX_ATTEMPTS),
        )
        .unwrap_or_else(|e| panic!("Failed to discover remote services:\n{}", e));

        T::sync(service_a.service_hash().as_str().to_string(), TIMEOUT);

        // Create a subscribe to connect to the tunneled service
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b = node_b
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let subscriber_b = service_b.subscriber_builder().create().unwrap();

        // === TEST ===
        for i in 0..num {
            // Publish
            let user_header_sent_at_a = MyHeader {
                version: 0,
                timestamp: 1000000000 + i as u64,
            };
            let mut payload_sent_at_a = String::with_capacity(PAYLOAD_DATA_LENGTH);
            for j in 0..PAYLOAD_DATA_LENGTH {
                let char_index = ((i * 7 + j * 13) % 26) as u8;
                let char_value = (b'A' + char_index) as char;
                payload_sent_at_a.push(char_value);
            }

            let mut sample_sent_at_a = publisher_a.loan_slice_uninit(PAYLOAD_DATA_LENGTH).unwrap();
            *sample_sent_at_a.user_header_mut() = user_header_sent_at_a.clone();
            let sample_sent_at_a = sample_sent_at_a.write_from_slice(payload_sent_at_a.as_bytes());
            sample_sent_at_a.send().unwrap();

            tunnel_a.propagate().unwrap();
            tunnel_b.propagate().unwrap();

            // Sync backends will see the sample on the first attempt; async
            // backends drive delivery forward via the in-closure propagate.
            T::retry(
                || match subscriber_b.receive().unwrap() {
                    Some(sample_received_at_b) => {
                        let user_header_received_at_b = sample_received_at_b.user_header();
                        let payload_received_at_b = sample_received_at_b.payload();

                        if *user_header_received_at_b != user_header_sent_at_a {
                            return Err("Failed to receive user header");
                        }
                        if *payload_received_at_b != *payload_sent_at_a.as_bytes() {
                            return Err("Failed to receive payload");
                        };

                        Ok(())
                    }
                    None => {
                        tunnel_a.propagate().unwrap();
                        tunnel_b.propagate().unwrap();
                        Err("Failed to receive sample")
                    }
                },
                TIMEOUT,
                Some(MAX_ATTEMPTS),
            )
            .unwrap_or_else(|e| panic!("Failed to propagate over tunnel:\n{}", e));
        }
    }

    #[conformance_test]
    pub fn propagates_struct_payload<S: Service, B: Backend<S> + Debug, T: Testing>() {
        propagate_struct_payloads::<S, B, T>(1);
    }

    #[conformance_test]
    pub fn propagates_struct_payload_many<S: Service, B: Backend<S> + Debug, T: Testing>() {
        propagate_struct_payloads::<S, B, T>(10);
    }

    #[conformance_test]
    pub fn propagates_slice_payload<S: Service, B: Backend<S> + Debug, T: Testing>() {
        propagate_slice_payloads::<S, B, T>(1);
    }

    #[conformance_test]
    pub fn propagates_slice_payload_many<S: Service, B: Backend<S> + Debug, T: Testing>() {
        propagate_slice_payloads::<S, B, T>(10);
    }

    #[conformance_test]
    pub fn samples_are_routed_to_their_own_service<
        S: Service,
        B: Backend<S> + Debug,
        T: Testing,
    >() {
        const MAX_ATTEMPTS: usize = 25;
        const TIMEOUT: Duration = Duration::from_millis(250);

        // === SETUP ===
        let service_name_1 = generate_service_name();
        let service_name_2 = generate_service_name();

        // --- Host A: two services, one publisher each ---
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
            .publish_subscribe::<MyType>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let publisher_a1 = service_a1.publisher_builder().create().unwrap();

        let service_a2 = node_a
            .service_builder(&service_name_2)
            .publish_subscribe::<MyType>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let publisher_a2 = service_a2.publisher_builder().create().unwrap();

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

        // Subscribers on host B
        let node_b = NodeBuilder::new()
            .config(&iceoryx_config_b)
            .create::<S>()
            .unwrap();
        let service_b1 = node_b
            .service_builder(&service_name_1)
            .publish_subscribe::<MyType>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let subscriber_b1 = service_b1.subscriber_builder().create().unwrap();

        let service_b2 = node_b
            .service_builder(&service_name_2)
            .publish_subscribe::<MyType>()
            .user_header::<MyHeader>()
            .open_or_create()
            .unwrap();
        let subscriber_b2 = service_b2.subscriber_builder().create().unwrap();

        // === TEST ===
        // Distinct payloads per service.
        let header_for_service_1 = MyHeader {
            version: 1,
            timestamp: 100,
        };
        let header_for_service_2 = MyHeader {
            version: 2,
            timestamp: 200,
        };
        let payload_for_service_1 = MyType {
            id: 1,
            value: 1.0,
            active: true,
        };
        let payload_for_service_2 = MyType {
            id: 2,
            value: 2.0,
            active: false,
        };

        let mut s1 = publisher_a1.loan_uninit().unwrap();
        *s1.user_header_mut() = header_for_service_1.clone();
        let s1 = s1.write_payload(payload_for_service_1.clone());
        s1.send().unwrap();

        let mut s2 = publisher_a2.loan_uninit().unwrap();
        *s2.user_header_mut() = header_for_service_2.clone();
        let s2 = s2.write_payload(payload_for_service_2.clone());
        s2.send().unwrap();

        tunnel_a.propagate().unwrap();
        tunnel_b.propagate().unwrap();

        // Each subscriber must receive the sample published on the corresponding service.
        for (label, subscriber, expected_header, expected_payload) in [
            (
                "service_1",
                &subscriber_b1,
                &header_for_service_1,
                &payload_for_service_1,
            ),
            (
                "service_2",
                &subscriber_b2,
                &header_for_service_2,
                &payload_for_service_2,
            ),
        ] {
            T::retry(
                || match subscriber.receive().unwrap() {
                    Some(received) => {
                        if *received.user_header() != *expected_header {
                            test_fail!("{}: received header from a different service", label);
                        }
                        if *received.payload() != *expected_payload {
                            test_fail!("{}: received payload from a different service", label);
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

    #[conformance_test]
    pub fn propagated_payloads_do_not_loop_back<S: Service, B: Backend<S> + Debug, T: Testing>() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

        // === SETUP ===
        let service_name = generate_service_name();

        let iceoryx_config = generate_isolated_config();
        let mut tunnel = Tunnel::<S, B>::new()
            .iceoryx_config(iceoryx_config.clone())
            .polled()
            .create()
            .unwrap();

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
        assert_that!(tunnel.tunneled_services().contains(service.service_hash()), eq true);

        // ==================== TEST =====================

        // [[ HOST A ]]
        // Publish
        let sample = publisher.loan_slice_uninit(PAYLOAD_DATA.len()).unwrap();
        let sample = sample.write_from_slice(PAYLOAD_DATA.as_bytes());
        sample.send().unwrap();

        // Receive - Sample should be received from local publisher
        while let Ok(Some(_)) = subscriber.receive() {}

        // Propagate
        tunnel.propagate().unwrap();

        // Receive - Sample should not loop back and be received again
        if subscriber.receive().unwrap().is_some() {
            test_fail!("sample looped back")
        }
    }
}
