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
mod service_discovery_tracker {

    use iceoryx2::prelude::*;
    use iceoryx2::service::service_hash::ServiceHash;
    use iceoryx2::testing::*;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_services_discovery::service_discovery::{Tracker, TrackerEvent};

    fn collect_sync<S: Service>(tracker: &mut Tracker<S>) -> (Vec<ServiceHash>, Vec<ServiceHash>) {
        let mut added: Vec<ServiceHash> = vec![];
        let mut removed: Vec<ServiceHash> = vec![];
        tracker
            .sync(|event| match event {
                TrackerEvent::Added(d) => added.push(*d.static_details.service_hash()),
                TrackerEvent::Removed(d) => removed.push(*d.static_details.service_hash()),
            })
            .expect("failed to sync tracker");
        (added, removed)
    }

    #[test]
    fn syncs_added_and_removed_publish_subscribe_services<S: Service>() {
        const NUMBER_OF_SERVICES_ADDED: usize = 8;
        const NUMBER_OF_SERVICES_REMOVED: usize = 3;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let mut sut = Tracker::<S>::new(&config);

        // Add a bunch of services.
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

        // Initial sync reports every service as added; nothing removed.
        let (added, removed) = collect_sync(&mut sut);
        assert_that!(added.len(), eq NUMBER_OF_SERVICES_ADDED);
        assert_that!(removed.len(), eq 0);
        for service in &services {
            assert_that!(added, contains * service.service_hash());
        }

        // A follow-up sync with no system changes reports nothing.
        let (added, removed) = collect_sync(&mut sut);
        assert_that!(added.len(), eq 0);
        assert_that!(removed.len(), eq 0);

        // Drop a subset; the next sync reports them as removed.
        let mut dropped_hashes = vec![];
        for _ in 0..NUMBER_OF_SERVICES_REMOVED {
            let service = services.pop().unwrap();
            dropped_hashes.push(*service.service_hash());
            drop(service);
        }
        let (added, removed) = collect_sync(&mut sut);
        assert_that!(added.len(), eq 0);
        assert_that!(removed.len(), eq NUMBER_OF_SERVICES_REMOVED);
        for hash in &removed {
            assert_that!(dropped_hashes, contains * hash);
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}

#[cfg(unix)]
mod failed_ipc_listing {
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
    };

    use iceoryx2::config::Config;
    use iceoryx2::prelude::*;
    use iceoryx2::service::{ServiceDetailsError, ServiceListError, service_hash::ServiceHash};
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_services_discovery::service_discovery::{SyncError, Tracker, TrackerEvent};

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

    #[test]
    fn failed_scan_keeps_unseen_services_then_successful_scan_removes_absent_one() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new()
            .config(&config)
            .create::<ipc::Service>()
            .expect("create node");
        let absent = node
            .service_builder(&generate_service_name())
            .event()
            .create()
            .expect("create service");
        let damaged = node
            .service_builder(&generate_service_name())
            .event()
            .create()
            .expect("create service");
        let absent_hash = *absent.service_hash();
        let damaged_hash = *damaged.service_hash();
        let mut tracker = Tracker::<ipc::Service>::new(&config);
        let mut added = 0;
        tracker
            .sync(|event| {
                if matches!(event, TrackerEvent::Added(_)) {
                    added += 1;
                }
            })
            .expect("initial scan");
        assert_eq!(added, 2);

        drop(absent);
        let damaged_config = CorruptedStaticConfig::new(&config, &damaged_hash);
        let mut removed = 0;
        assert_eq!(
            tracker.sync(|event| {
                if matches!(event, TrackerEvent::Removed(_)) {
                    removed += 1;
                }
            }),
            Err(SyncError::ServiceLookupFailure)
        );
        assert_eq!(removed, 0);
        assert!(tracker.get(&absent_hash).is_some());
        assert!(tracker.get(&damaged_hash).is_some());

        drop(damaged_config);
        let mut removed_hashes = Vec::new();
        tracker
            .sync(|event| {
                if let TrackerEvent::Removed(details) = event {
                    removed_hashes.push(*details.static_details.service_hash());
                }
            })
            .expect("recovered scan");
        assert_eq!(removed_hashes, vec![absent_hash]);
        assert!(tracker.get(&absent_hash).is_none());
        assert!(tracker.get(&damaged_hash).is_some());
    }

    #[test]
    fn nested_detail_permission_failure_stays_a_permission_error() {
        let hash = ServiceHash::try_from("0123456789abcdef").expect("valid hash");
        assert_eq!(
            SyncError::from(ServiceListError::FailedToAcquireServiceDetails {
                service_hash: hash,
                error: ServiceDetailsError::InsufficientPermissions,
            }),
            SyncError::InsufficientPermissions
        );
    }
}
