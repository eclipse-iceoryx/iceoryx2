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
mod zenoh_tunnel {

    use std::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::service::static_config::StaticConfig;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_tunnels_zenoh::*;

    use zenoh::Wait;

    fn mock_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_tunnel_zenoh_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn discovers_local_services<S: Service>() {
        // create tunnel
        let iox_config = generate_isolated_config();
        let mut tunnel = Tunnel::<S>::new(&iox_config);
        tunnel.initialize();
        assert_that!(tunnel.tunneled_services().len(), eq 0);

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service_name = mock_service_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        tunnel.discover();

        // verify stream is set up for the created service
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel
            .tunneled_services()
            .contains(&String::from(iox_service.service_id().as_str())), eq true);
    }

    #[test]
    fn discovers_remote_services<S: Service>() {
        // create tunnel on host a
        let iox_config_a = generate_isolated_config();
        let mut tunnel_a = Tunnel::<S>::new(&iox_config_a);
        tunnel_a.initialize();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        // create tunnel on host b
        let iox_config_b = generate_isolated_config();
        let mut tunnel_b = Tunnel::<S>::new(&iox_config_b);
        tunnel_b.initialize();
        assert_that!(tunnel_b.tunneled_services().len(), eq 0);

        // create an iceoryx2 service on host b
        let iox_node_b = NodeBuilder::new()
            .config(&iox_config_b)
            .create::<S>()
            .unwrap();
        let iox_service_name_b = mock_service_name();
        let _iox_service_b = iox_node_b
            .service_builder(&iox_service_name_b)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // run discovery in tunnel a
        //  the service on host b should not be discovered
        tunnel_a.discover();
        assert_that!(tunnel_a.tunneled_services().len(), eq 0);

        // run discovery in tunnel b
        //  the service on host b should be discovered and announced
        //  by tunnel b
        tunnel_b.discover();
        assert_that!(tunnel_b.tunneled_services().len(), eq 1);

        // run discovery in tunnel a
        //  the service on host b should now be discovered on host a
        let mut success = false;
        for _ in 0..3 {
            tunnel_a.discover();
            if tunnel_a.tunneled_services().len() == 1 {
                success = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if !success {
            test_fail!("failed to discover remote service after 3 attempts");
        }
    }

    #[test]
    fn propagates_data_from_local_publishers_to_remote_hosts<S: Service>() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

        // create tunnel
        let iox_config = generate_isolated_config();
        let mut tunnel = Tunnel::<S>::new(&iox_config);
        tunnel.initialize();

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service_name = mock_service_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        tunnel.discover();

        // create iceoryx2 publisher
        let iox_publisher = iox_service
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA.len())
            .create()
            .unwrap();

        // create zenoh subscriber
        let z_config = zenoh::config::Config::default();
        let z_session = zenoh::open(z_config.clone()).wait().unwrap();
        let z_subscriber = z_session
            .declare_subscriber(keys::data_stream(iox_service.service_id()))
            .wait()
            .unwrap();

        // send data on iceoryx2 publisher
        let sample = iox_publisher.loan_slice_uninit(PAYLOAD_DATA.len()).unwrap();
        let sample = sample.write_from_slice(PAYLOAD_DATA.as_bytes());
        sample.send().unwrap();

        // propagate over tunnel
        tunnel.propagate();

        // receive data on zenoh subscriber
        if let Ok(Some(z_sample)) = z_subscriber.recv_timeout(Duration::from_millis(100)) {
            let z_payload = z_sample.payload().try_to_string().unwrap();
            assert_that!(z_payload, eq PAYLOAD_DATA);
        } else {
            test_fail!("payload was not propagated from iceoryx2 to Zenoh")
        }
    }

    #[test]
    fn prevents_loopback_of_data_published_to_remote_hosts<S: Service>() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

        // create tunnel
        let iox_config = generate_isolated_config();
        let mut tunnel = Tunnel::<S>::new(&iox_config);
        tunnel.initialize();

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service_name = mock_service_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        tunnel.discover();

        // create iceoryx2 publisher
        let iox_publisher = iox_service
            .publisher_builder()
            .initial_max_slice_len(PAYLOAD_DATA.len())
            .create()
            .unwrap();

        // create iceoryx2 subscriber
        let iox_subscriber = iox_service.subscriber_builder().create().unwrap();

        // send data on iceoryx2 publisher
        let sample = iox_publisher.loan_slice_uninit(PAYLOAD_DATA.len()).unwrap();
        let sample = sample.write_from_slice(PAYLOAD_DATA.as_bytes());
        sample.send().unwrap();

        // drain iceoryx2 subscriber receive queue
        while let Ok(Some(_)) = iox_subscriber.receive() {}

        // propagate over tunnel
        tunnel.propagate();

        // ensure no loopback to iceoryx subscribe
        // -> detectedable by empty subscriber queue
        if iox_subscriber.receive().unwrap().is_some() {
            test_fail!("sample looped back")
        }
    }

    #[test]
    fn propagates_data_from_remote_hosts_to_local_subscribers<S: Service>() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

        // create tunnel
        let iox_config = generate_isolated_config();
        let mut tunnel = Tunnel::<S>::new(&iox_config);
        tunnel.initialize();

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service_name = mock_service_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        //  service should be discovered
        tunnel.discover();

        // create iceoryx2 subscriber
        let iox_subscriber = iox_service.subscriber_builder().create().unwrap();

        // create zenoh publisher
        let z_config = zenoh::config::Config::default();
        let z_session = zenoh::open(z_config.clone()).wait().unwrap();
        let z_publisher = z_session
            .declare_publisher(keys::data_stream(iox_service.service_id()))
            .wait()
            .unwrap();

        // send data on zenoh publisher
        z_publisher.put(PAYLOAD_DATA.as_bytes()).wait().unwrap();
        std::thread::sleep(Duration::from_millis(500)); // wait for zenoh background thread ...

        // propagate over tunnel
        tunnel.propagate();

        // verify data received at iceoryx subscriber
        let sample = iox_subscriber.receive().unwrap();
        assert_that!(sample, is_some);
        let sample = sample.unwrap();

        // Convert payload to a string safely
        let bytes = sample.payload();
        match std::str::from_utf8(bytes) {
            Ok(payload_str) => {
                assert_that!(payload_str, eq PAYLOAD_DATA);
            }
            Err(_) => test_fail!("Payload is not valid UTF-8"),
        }
    }

    #[test]
    fn responds_to_zenoh_query_for_details_of_local_services<S: Service>() {
        let iox_config = generate_isolated_config();

        // create tunnel
        let mut tunnel = Tunnel::<S>::new(&iox_config);
        tunnel.initialize();

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<S>()
            .unwrap();
        let iox_service_name = mock_service_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        tunnel.discover();

        // verify static config is retrievable from zenoh
        let z_config = zenoh::config::Config::default();
        let z_session = zenoh::open(z_config.clone()).wait().unwrap();
        let z_reply = z_session
            .get(keys::service(iox_service.service_id()))
            .wait()
            .unwrap();
        match z_reply.recv_timeout(Duration::from_millis(500)) {
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

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
