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
