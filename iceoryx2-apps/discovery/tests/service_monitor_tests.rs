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

mod service_monitor {

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_discovery::service::{Monitor, MonitorConfig};

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_monitor_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn publishes_added_services_when_configured() {
        const NUMBER_OF_SERVICES_ADDED: usize = 3;
        const TEST_SERVICE_MONITOR_NAME: &str =
            "iox2://test/publishes_added_services_when_configured";

        let iceoryx_config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        let monitor_config = MonitorConfig {
            service_name: TEST_SERVICE_MONITOR_NAME.to_string(),
            ignore_internal: true,
            publish_events: true,
            send_notifications: false,
        };
        let mut sut = Monitor::<ipc::Service>::new(&monitor_config, &iceoryx_config);
        sut.spin();

        // subscribe to the monitoring service
        let service_name = ServiceName::new(TEST_SERVICE_MONITOR_NAME).unwrap();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<iceoryx2_discovery::service::DiscoveryEvent>()
            .open()
            .unwrap();
        let subscriber = service.subscriber_builder().create().unwrap();

        // add some services
        let mut services = vec![];
        for _ in 0..NUMBER_OF_SERVICES_ADDED {
            let service_name = generate_name();
            let service = node
                .service_builder(&service_name)
                .publish_subscribe::<u64>()
                .create()
                .unwrap();
            services.push(service);
        }

        // verify the added services are published on next monitor iteration
        sut.spin();

        let mut num_received = 0;
        while let Ok(Some(event)) = subscriber.receive() {
            match event.payload() {
                iceoryx2_discovery::service::DiscoveryEvent::Added(service) => {
                    println!("added {:?}", service.name())
                }
                iceoryx2_discovery::service::DiscoveryEvent::Removed(service) => {
                    println!("removed {}", service.name())
                }
            }
            num_received += 1;
        }

        assert_that!(num_received, eq NUMBER_OF_SERVICES_ADDED);
    }

    #[test]
    fn publishes_removed_services_when_configured() {}

    #[test]
    fn sends_notifications_when_configured() {}

    #[test]
    fn monitors_internal_services_when_configured() {}
}
