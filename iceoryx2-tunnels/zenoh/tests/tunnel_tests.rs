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

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_tunnel_zenoh_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn propagates_data_from_local_services_to_remote_hosts() {
        const PAYLOAD_DATA: &str = "WhenItRegisters";

        let iox_config = generate_isolated_config();

        // create tunnel
        let mut sut = ZenohTunnel::new(iox_config.clone());
        sut.initialize();

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<ipc::Service>()
            .unwrap();
        let iox_service_name = generate_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        sut.discover();

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
        sut.propagate();

        // receive data on zenoh subscriber
        if let Ok(Some(sample)) = z_subscriber.recv_timeout(Duration::from_millis(500)) {
            let received_data = sample.payload().try_to_string().unwrap();
            assert_that!(received_data, eq PAYLOAD_DATA);
        } else {
            test_fail!("payload was not propagated from iceoryx2 to Zenoh")
        }
    }

    #[test]
    fn responds_to_zenoh_query_for_service_type_details() {
        let iox_config = generate_isolated_config();

        // create tunnel
        let mut sut = ZenohTunnel::new(iox_config.clone());
        sut.initialize();

        // create iceoryx2 service
        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<ipc::Service>()
            .unwrap();
        let iox_service_name = generate_name();
        let iox_service = iox_node
            .service_builder(&iox_service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();

        // discover iceoryx2 service
        sut.discover();

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
                    let z_static_details: StaticConfig =
                        serde_json::from_slice(&sample.payload().to_bytes()).unwrap();
                    assert_that!(z_static_details.name().as_str(), eq iox_service_name.as_str());
                    assert_that!(z_static_details.publish_subscribe().message_type_details(), eq iox_service.static_config().message_type_details());
                }
                Err(e) => test_fail!("error reading reply to type details query: {}", e),
            },
            Ok(None) => test_fail!("no reply to type details query"),
            Err(e) => test_fail!("error querying message type details from zenoh: {}", e),
        }
    }
}
