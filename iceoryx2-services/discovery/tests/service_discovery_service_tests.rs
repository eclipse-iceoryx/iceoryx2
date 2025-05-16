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

mod service_discovery_service {

    use iceoryx2::prelude::*;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_services_discovery::service_discovery::{
        service_name, Config, Discovery, Payload, Service,
    };

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_service_monitor_service_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn publishes_details_of_added_and_removed_services_when_configured() {
        const NUMBER_OF_SERVICES_ADDED: usize = 5;
        const NUMBER_OF_SERVICES_REMOVED: usize = 3;

        let iceoryx_config = generate_isolated_config();

        // create a service monitoring service
        let discovery_config = Config {
            sync_on_initialization: true,
            include_internal: false,
            publish_events: true,
            max_subscribers: 1,
            send_notifications: false,
            max_listeners: 1,
            ..Default::default()
        };
        let mut sut = Service::<ipc::Service>::create(&discovery_config, &iceoryx_config).unwrap();

        // subscribe to the monitoring service
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        let service = node
            .service_builder(service_name())
            .publish_subscribe::<Payload>()
            .open_or_create()
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
        sut.spin(|_| {}, |_| {}).unwrap();

        // remove some services
        for _ in 0..NUMBER_OF_SERVICES_REMOVED {
            services.pop();
        }
        sut.spin(|_| {}, |_| {}).unwrap();

        let mut num_added = 0;
        let mut num_removed = 0;
        while let Ok(Some(sample)) = subscriber.receive() {
            match sample.payload() {
                Discovery::Added(_) => {
                    num_added += 1;
                }
                Discovery::Removed(_) => {
                    num_removed += 1;
                }
            }
        }

        assert_that!(num_added, eq NUMBER_OF_SERVICES_ADDED);
        assert_that!(num_removed, eq NUMBER_OF_SERVICES_REMOVED);
    }

    #[test]
    fn sends_events_for_added_or_removed_services_when_configured() {
        let iceoryx_config = generate_isolated_config();

        // create a service monitoring service
        let discovery_config = Config {
            sync_on_initialization: true,
            include_internal: false,
            publish_events: false,
            max_subscribers: 1,
            send_notifications: true,
            max_listeners: 1,
            ..Default::default()
        };
        let mut sut = Service::<ipc::Service>::create(&discovery_config, &iceoryx_config).unwrap();

        // listen to the monitoring service
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        let service = node
            .service_builder(service_name())
            .event()
            .open_or_create()
            .unwrap();
        let listener = service.listener_builder().create().unwrap();

        // add a service
        let service_name = generate_name();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();
        sut.spin(|_| {}, |_| {}).unwrap();

        let result = listener.try_wait_one();
        assert_that!(result, is_ok);
        let result = result.unwrap();
        assert_that!(result, is_some);

        // remove a service
        drop(service);
        sut.spin(|_| {}, |_| {}).unwrap();

        let result = listener.try_wait_one();
        assert_that!(result, is_ok);
        let result = result.unwrap();
        assert_that!(result, is_some);
    }

    #[test]
    fn monitors_internal_services_when_configured() {
        let iceoryx_config = generate_isolated_config();

        // create a service monitoring service
        let discovery_config = Config {
            sync_on_initialization: false,
            include_internal: true,
            publish_events: true,
            max_subscribers: 1,
            send_notifications: false,
            max_listeners: 1,
            ..Default::default()
        };
        let mut sut = Service::<ipc::Service>::create(&discovery_config, &iceoryx_config).unwrap();

        // subscribe to the monitoring service
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        let service = node
            .service_builder(service_name())
            .publish_subscribe::<Payload>()
            .open_or_create()
            .unwrap();
        let subscriber = service.subscriber_builder().create().unwrap();

        // check for service changes
        sut.spin(|_| {}, |_| {}).unwrap();

        // verify the addition of this service is announced (as it is an internal service)
        let result = subscriber.receive();
        assert_that!(result, is_ok);
        let result = result.unwrap();
        assert_that!(result, is_some);
        let sample = result.unwrap();

        if let Discovery::Added(service_info) = sample.payload() {
            assert_that!(service_info.name().to_string(), eq service_name().as_str());
        } else {
            test_fail!("expected DiscoveryEvent::Added for the internal service")
        }
    }
}
