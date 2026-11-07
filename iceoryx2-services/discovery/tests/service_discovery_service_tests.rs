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

extern crate alloc;

mod service_discovery_service {

    use iceoryx2::prelude::*;
    use iceoryx2::service::static_config::StaticConfig;
    use iceoryx2::testing::generate_service_name;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::test_fail;
    use iceoryx2_services_discovery::service_discovery::{
        Config, DiscoveryEvent, Service, service_name,
    };

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
            max_buffer_size: 10,
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
            .publish_subscribe::<DiscoveryEvent>()
            .open_or_create()
            .unwrap();
        let subscriber = service.subscriber_builder().create().unwrap();

        // add some services
        let mut services = vec![];
        for _ in 0..NUMBER_OF_SERVICES_ADDED {
            let service_name = generate_service_name();
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
        let service_name = generate_service_name();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();
        sut.spin(|_| {}, |_| {}).unwrap();

        let number_of_notifications = listener.try_wait(|_| {}).unwrap();
        assert_that!(number_of_notifications, eq 1);

        // remove a service
        drop(service);
        sut.spin(|_| {}, |_| {}).unwrap();

        let number_of_notifications = listener.try_wait(|_| {}).unwrap();
        assert_that!(number_of_notifications, eq 1);
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
            .publish_subscribe::<DiscoveryEvent>()
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

        if let DiscoveryEvent::Added(service_info) = sample.payload() {
            assert_that!(service_info.name().to_string(), eq service_name().as_str());
        } else {
            test_fail!("expected DiscoveryEvent::Added for the internal service")
        }
    }

    #[test]
    fn get_current_discovery_states() -> Result<(), Box<dyn core::error::Error>> {
        const NUM_SERVICES: usize = 100;

        let iceoryx_config = generate_isolated_config();

        // create some services in the main thread
        let node = NodeBuilder::new()
            .config(&iceoryx_config)
            .create::<ipc::Service>()
            .unwrap();

        let service_names: Vec<_> = (0..NUM_SERVICES).map(|_| generate_service_name()).collect();

        let _created_services: Vec<_> = service_names
            .iter()
            .map(|name| {
                node.service_builder(name)
                    .publish_subscribe::<u64>()
                    .create()
                    .unwrap()
            })
            .collect();

        let discovery_config = Config {
            sync_on_initialization: true,
            include_internal: false,
            publish_events: true,
            max_subscribers: 1,
            send_notifications: false,
            max_listeners: 1,
            initial_max_slice_len: NUM_SERVICES,
            ..Default::default()
        };

        let mut sut = Service::<ipc::Service>::create(&discovery_config, &iceoryx_config).unwrap();

        // === Request current discovery state ===
        let service = node
            .service_builder(service_name())
            .request_response::<(), [StaticConfig]>()
            .open_or_create()
            .unwrap();

        let client = service.client_builder().create().unwrap();

        let pending_response = client.loan_uninit()?.write_payload(()).send()?;

        let mut counter = 0;

        sut.spin(|_| {}, |_| {})?;

        while let Some(response) = pending_response.receive()? {
            for service in response.payload().iter() {
                assert_that!(service_names.contains(service.name()), eq true);
                counter += 1;
            }
        }

        assert_that!(counter, eq service_names.len());

        Ok(())
    }

    #[test]
    fn cleanup_removes_node_resources() {
        let iceoryx_config = generate_isolated_config();
        let sut = Service::<ipc::Service>::create(&Config::default(), &iceoryx_config).unwrap();

        let mut node_id = None;
        Node::<ipc::Service>::list(&iceoryx_config, |node_state| {
            node_id = Some(*node_state.node_id());
            CallbackProgression::Stop
        })
        .unwrap();
        let node_id = node_id.unwrap();

        assert_that!(iceoryx2::testing::do_stale_node_resources_exist::<ipc::Service>(&iceoryx_config, node_id), eq true);
        drop(sut);
        assert_that!(iceoryx2::testing::do_stale_node_resources_exist::<ipc::Service>(&iceoryx_config, node_id), eq false);
    }
}

#[cfg(unix)]
mod failed_ipc_listing {
    use iceoryx2::config::Config;
    use iceoryx2::prelude::*;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_services_discovery::service_discovery::{
        self, DiscoveryEvent, Service, SpinError, service_name,
    };
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
    };
    fn static_config_path(config: &Config, hash: &ServiceHash) -> PathBuf {
        PathBuf::from(config.global.root_path().as_str())
            .join(config.global.service.directory.as_str())
            .join(format!(
                "{}{}{}",
                config.global.prefix, hash, config.global.service.static_config_storage_suffix
            ))
    }

    fn write_static_config(path: &Path, contents: &[u8]) {
        let original_permissions = fs::metadata(path)
            .expect("stat static config")
            .permissions();
        let mut writable_permissions = original_permissions.clone();
        writable_permissions.set_mode(original_permissions.mode() | 0o200);
        fs::set_permissions(path, writable_permissions).expect("make static config writable");
        let write_result = fs::write(path, contents);
        let restore_result = fs::set_permissions(path, original_permissions);
        restore_result.expect("restore static config permissions");
        write_result.expect("write static config");
    }

    struct CorruptedStaticConfig {
        path: PathBuf,
        original: Vec<u8>,
    }

    impl CorruptedStaticConfig {
        fn new(config: &Config, hash: &ServiceHash) -> Self {
            let path = static_config_path(config, hash);
            let original = fs::read(&path).expect("read static config");
            write_static_config(&path, b"invalid static service config");
            Self { path, original }
        }
    }

    impl Drop for CorruptedStaticConfig {
        fn drop(&mut self) {
            write_static_config(&self.path, &self.original);
        }
    }

    fn partial_scan_notifies(publish_events: bool) {
        let config = generate_isolated_config();
        let discovery_config = service_discovery::Config {
            sync_on_initialization: true,
            include_internal: false,
            publish_events,
            send_notifications: true,
            max_listeners: 1,
            max_subscribers: 1,
            max_buffer_size: 10,
            ..Default::default()
        };
        let mut sut = Service::<ipc::Service>::create(&discovery_config, &config).unwrap();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<ipc::Service>()
            .unwrap();
        let notifications = node.service_builder(service_name()).event().open().unwrap();
        let listener = notifications.listener_builder().create().unwrap();
        let events = if publish_events {
            Some(
                node.service_builder(service_name())
                    .publish_subscribe::<DiscoveryEvent>()
                    .open()
                    .unwrap(),
            )
        } else {
            None
        };
        let subscriber = events
            .as_ref()
            .map(|s| s.subscriber_builder().create().unwrap());
        sut.spin(|_| {}, |_| {}).unwrap();
        listener.try_wait(|_| {}).unwrap();
        let first = node
            .service_builder(&generate_service_name())
            .event()
            .create()
            .unwrap();
        let second = node
            .service_builder(&generate_service_name())
            .event()
            .create()
            .unwrap();
        let mut corrupted = None;
        let mut added = Vec::new();
        // Corrupt the other service only after the first was discovered, independently
        // of the directory enumeration order. Its details have not been read yet.
        let result = sut.spin(
            |details| {
                let hash = *details.static_details.service_hash();
                added.push(hash);
                let other = if hash == *first.service_hash() {
                    second.service_hash()
                } else {
                    first.service_hash()
                };
                corrupted = Some(CorruptedStaticConfig::new(&config, other));
            },
            |_| panic!("incomplete scan must not remove services"),
        );
        assert_eq!(result, Err(SpinError::SyncFailure));
        assert_eq!(added.len(), 1);
        assert_eq!(listener.try_wait(|_| {}).unwrap(), 1);
        if let Some(subscriber) = &subscriber {
            let sample = subscriber
                .receive()
                .unwrap()
                .expect("published partial addition");
            assert!(
                matches!(sample.payload(), DiscoveryEvent::Added(details) if *details.service_hash() == added[0])
            );
            assert!(subscriber.receive().unwrap().is_none());
        }
        drop(corrupted);
        let mut recovered = Vec::new();
        sut.spin(
            |details| recovered.push(*details.static_details.service_hash()),
            |_| panic!("no removals expected"),
        )
        .unwrap();
        assert_eq!(recovered.len(), 1);
        assert_ne!(recovered[0], added[0]);
        assert_eq!(listener.try_wait(|_| {}).unwrap(), 1);
        sut.spin(
            |_| panic!("no duplicate additions"),
            |_| panic!("no removals expected"),
        )
        .unwrap();
        assert_eq!(listener.try_wait(|_| {}).unwrap(), 0);
    }

    #[test]
    fn partial_scan_notifies_without_publishing_events() {
        partial_scan_notifies(false);
    }

    #[test]
    fn partial_scan_notifies_with_published_events() {
        partial_scan_notifies(true);
    }
}
