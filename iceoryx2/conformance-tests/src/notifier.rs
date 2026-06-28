// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod notifier {
    use alloc::collections::BTreeSet;
    use alloc::{format, vec};

    use iceoryx2::{
        node::NodeBuilder,
        port::notifier::{NotifierCreateError, NotifierNotifyError},
        port::port_name::PortName,
        service::Service,
    };
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_testing::*;

    #[conformance_test]
    pub fn create_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", NotifierCreateError::ExceedsMaxSupportedNotifiers), eq "NotifierCreateError::ExceedsMaxSupportedNotifiers");
    }

    #[conformance_test]
    pub fn notify_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", NotifierNotifyError::EventIdOutOfBounds), eq "NotifierNotifyError::EventIdOutOfBounds");
    }

    #[conformance_test]
    pub fn id_is_unique<Sut: Service>() {
        let test = Test::<Sut>::new();
        let service_name = generate_service_name();
        let node = NodeBuilder::new()
            .config(test.config())
            .create::<Sut>()
            .unwrap();
        const MAX_LISTENERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(MAX_LISTENERS)
            .create()
            .unwrap();

        let mut listeners = vec![];
        let mut listener_id_set = BTreeSet::new();

        for _ in 0..MAX_LISTENERS {
            let listener = sut.listener_builder().create().unwrap();
            assert_that!(listener_id_set.insert(listener.id()), eq true);
            listeners.push(listener);
        }
    }

    #[conformance_test]
    pub fn notifier_name_is_empty_by_default<Sut: Service>()
    -> core::result::Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        let test = Test::<Sut>::new();
        let service_name = generate_service_name();
        let node = test.create_node();
        let service = node.service_builder(&service_name).event().create()?;

        let sut = service.notifier_builder().create()?;

        assert_that!(sut.name(), eq "");

        Ok(())
    }

    #[conformance_test]
    pub fn notifier_name_can_be_set<Sut: Service>()
    -> core::result::Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        let test = Test::<Sut>::new();
        let service_name = generate_service_name();
        let node = test.create_node();
        let service = node.service_builder(&service_name).event().create()?;

        let notifier_name = PortName::new("yell").unwrap();
        let sut = service.notifier_builder().name(&notifier_name).create()?;

        assert_that!(sut.name(), eq notifier_name);

        Ok(())
    }
}
