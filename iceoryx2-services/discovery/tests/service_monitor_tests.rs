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
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_services_common::INTERNAL_SERVICE_PREFIX;
    use iceoryx2_services_discovery::service::{DiscoveryEvent, Monitor, MonitorConfig};

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_service_monitor_service_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn publishes_added_and_removed_services_when_configured() {
        const NUMBER_OF_SERVICES_ADDED: usize = 5;
        const NUMBER_OF_SERVICES_REMOVED: usize = 3;

        let iceoryx_config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        // create a service monitoring service
        let service_name_string: String = INTERNAL_SERVICE_PREFIX.to_owned()
            + "test/service_monitor/publishes_added_services_when_configured";
        let monitor_config = MonitorConfig {
            service_name: service_name_string.to_string(),
            include_internal: false,
            publish_events: true,
            send_notifications: false,
        };
        let mut sut = Monitor::<ipc::Service>::new(&monitor_config, &iceoryx_config);

        // subscribe to the monitoring service
        let service_name = ServiceName::new(service_name_string.as_str()).unwrap();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<DiscoveryEvent>()
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
        sut.spin();

        // remove some services
        for _ in 0..NUMBER_OF_SERVICES_REMOVED {
            services.pop();
        }
        sut.spin();

        let mut num_added = 0;
        let mut num_removed = 0;
        while let Ok(Some(event)) = subscriber.receive() {
            match event.payload() {
                DiscoveryEvent::Added(_) => {
                    num_added += 1;
                }
                DiscoveryEvent::Removed(_) => {
                    num_removed += 1;
                }
            }
        }

        assert_that!(num_added, eq NUMBER_OF_SERVICES_ADDED);
        assert_that!(num_removed, eq NUMBER_OF_SERVICES_REMOVED);
    }

    #[test]
    fn sends_notifications_when_configured() {}

    #[test]
    fn monitors_internal_services_when_configured() {
        let iceoryx_config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        // create a service monitoring service
        let service_name_string: String = INTERNAL_SERVICE_PREFIX.to_owned()
            + "test/service_monitor/monitors_internal_services_when_configured";
        let monitor_config = MonitorConfig {
            service_name: service_name_string.to_string(),
            include_internal: true,
            publish_events: true,
            send_notifications: false,
        };
        let mut sut = Monitor::<ipc::Service>::new(&monitor_config, &iceoryx_config);

        // subscribe to the monitoring service
        let service_name = ServiceName::new(service_name_string.as_str()).unwrap();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<DiscoveryEvent>()
            .open()
            .unwrap();
        let subscriber = service.subscriber_builder().create().unwrap();

        // check for service changes
        sut.spin();

        // verify the addition of this service is announced (as it is an internal service)
        let result = subscriber.receive();
        assert_that!(result, is_ok);
        let result = result.unwrap();
        assert_that!(result, is_some);
        let service = result.unwrap();
        if let DiscoveryEvent::Added(service_info) = service.payload() {
            assert_that!(service_info.name().to_string(), eq service_name_string);
        } else {
            test_fail!("expected DiscoveryEvent::Added for the internal service")
        }
    }
}
