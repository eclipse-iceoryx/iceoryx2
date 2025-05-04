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
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_services_discovery::service_discovery::Tracker;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "test_service_tracker_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn syncs_added_publish_subscribe_services<S: Service>() {
        const NUMBER_OF_SERVICES_ADDED: usize = 8;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let mut sut = Tracker::<S>::new();

        // add a bunch of services
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

        // verify added services are detected
        let (added, _) = sut.sync(&config).expect("failed to sync tracker");

        assert_that!(added.len(), eq NUMBER_OF_SERVICES_ADDED);
        for service in &services {
            assert_that!(added, contains * service.service_id());
        }

        // verify added services are not detected again in subsequent sync
        let (added, removed) = sut.sync(&config).expect("failed to sync tracker");
        assert_that!(added.len(), eq 0);
        assert_that!(removed.len(), eq 0);
    }

    #[test]
    fn syncs_removed_publish_subscribe_services<S: Service>() {
        const NUMBER_OF_SERVICES_ADDED: usize = 8;
        const NUMBER_OF_SERVICES_REMOVED: usize = 3;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let mut sut = Tracker::<S>::new();

        // add a bunch of services
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

        let (added, _) = sut.sync(&config).expect("failed to sync tracker");
        assert_that!(added.len(), eq NUMBER_OF_SERVICES_ADDED);

        // remove some services by dropping them
        let mut removed_ids = vec![];
        for _ in 0..NUMBER_OF_SERVICES_REMOVED {
            let removed = services.pop().unwrap();
            removed_ids.push(removed.service_id().clone());
            drop(removed);
        }

        // verify the dropped services are detected as removed
        let (_, removed) = sut.sync(&config).expect("failed to sync tracker");
        assert_that!(removed.len(), eq NUMBER_OF_SERVICES_REMOVED);
        for service in removed {
            assert_that!(removed_ids, contains * service.static_details.service_id());
        }
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
