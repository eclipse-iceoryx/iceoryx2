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
mod publish_subscribe_relay_tests {

    use std::time::Duration;

    use iceoryx2::prelude::*;
    use iceoryx2::service::static_config::StaticConfig;
    use iceoryx2::testing::*;

    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::{assert_that, test_fail};
    use iceoryx2_tunnel_backend::traits::Backend;
    use iceoryx2_tunnel_backend::traits::RelayBuilder;
    use iceoryx2_tunnel_backend::traits::RelayFactory;
    use iceoryx2_tunnel_zenoh::{keys, ZenohBackend};

    use zenoh::Wait;

    fn generate_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "publish_subscribe_relay_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn announces_relays_on_zenoh<S: Service>() {
        // ==================== SETUP ====================

        // === SETUP ===
        let service_name = generate_service_name();

        let iceoryx_config = generate_isolated_config();

        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<S>()
            .unwrap();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .history_size(10)
            .subscriber_max_buffer_size(10)
            .open_or_create()
            .unwrap();
        let static_config = S::details(
            &service_name,
            &iceoryx_config,
            MessagingPattern::PublishSubscribe,
        )
        .unwrap()
        .unwrap()
        .static_details;

        let zenoh_config = zenoh::Config::default();
        let backend = ZenohBackend::<S>::create(&zenoh_config).unwrap();

        // ==================== TEST =====================

        // Create a relay
        let _relay = backend
            .relay_builder()
            .publish_subscribe(&static_config)
            .create();

        // Query zenoh to verify service was announced
        let zenoh_config = zenoh::config::Config::default();
        let zenoh_session = zenoh::open(zenoh_config.clone()).wait().unwrap();
        let zenoh_reply = zenoh_session
            .get(keys::service_details(service.service_id()))
            .wait()
            .unwrap();
        match zenoh_reply.recv_timeout(Duration::from_millis(100)) {
            Ok(Some(reply)) => match reply.result() {
                Ok(sample) => {
                    let received_static_config: StaticConfig =
                        serde_json::from_slice(&sample.payload().to_bytes()).unwrap();
                    assert_that!(received_static_config.service_id(), eq service.service_id());
                    assert_that!(received_static_config.name(), eq & service_name);
                    assert_that!(received_static_config.publish_subscribe(), eq service.static_config());
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
