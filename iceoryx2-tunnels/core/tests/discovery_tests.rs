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

mod mocks;

#[generic_tests::define]
mod tunnel_discovery_tests {

    use crate::mocks::*;

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_tunnels_core::RelayFactory;
    use iceoryx2_tunnels_core::Transport;
    use iceoryx2_tunnels_core::Tunnel;

    // TODO: Move to iceoryx2::testing
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;

    fn generate_service_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_tunnel_discovery_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn discovers_services_via_tracker<S: Service, T: Transport>()
    where
        T: RelayFactory<T>,
    {
        // === SETUP ==
        let tunnel_config = iceoryx2_tunnels_core::Config::default();
        let iceoryx_config = generate_isolated_config();

        let service_name = generate_service_name();
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

        // === TEST ===
        let mut tunnel =
            Tunnel::<S, T>::create(&tunnel_config, &iceoryx_config, &T::Config::default()).unwrap();

        tunnel.discovery().unwrap();

        // === VALIDATE ===
        assert_that!(tunnel.tunneled_services().len(), eq 1);
        assert_that!(tunnel.tunneled_services().contains(service.service_id()), eq true);
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service, MockTransport>)]
    mod ipc {}
    #[instantiate_tests(<iceoryx2::service::local::Service, MockTransport>)]
    mod local {}
}
