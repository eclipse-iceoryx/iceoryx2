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
pub mod subscriber {
    use alloc::collections::BTreeSet;
    use alloc::{format, vec};
    use iceoryx2::port::ReceiveError;
    use iceoryx2::{
        port::port_name::PortName, port::subscriber::SubscriberCreateError, service::Service,
    };
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_testing::*;

    #[conformance_test]
    pub fn receive_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", ReceiveError::ExceedsMaxBorrows), eq "ReceiveError::ExceedsMaxBorrows");
    }

    #[conformance_test]
    pub fn create_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", SubscriberCreateError::ExceedsMaxSupportedSubscribers), eq "SubscriberCreateError::ExceedsMaxSupportedSubscribers");
        assert_that!(
            format!("{}", SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService), eq "SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService");
    }

    #[conformance_test]
    pub fn id_is_unique<Sut: Service>() {
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        const MAX_SUBSCRIBERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(MAX_SUBSCRIBERS)
            .create()
            .unwrap();

        let mut subscribers = vec![];
        let mut subscriber_id_set = BTreeSet::new();

        for _ in 0..MAX_SUBSCRIBERS {
            let subscriber = sut.subscriber_builder().create().unwrap();
            assert_that!(subscriber_id_set.insert(subscriber.id()), eq true);
            subscribers.push(subscriber);
        }
    }

    #[conformance_test]
    pub fn subscriber_name_is_empty_by_default<Sut: Service>()
    -> core::result::Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        let test = Test::<Sut>::new();
        let service_name = generate_service_name();
        let node = test.create_node();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.subscriber_builder().create()?;

        assert_that!(sut.name(), eq "");

        Ok(())
    }

    #[conformance_test]
    pub fn subscriber_name_can_be_set<Sut: Service>()
    -> core::result::Result<(), alloc::boxed::Box<dyn core::error::Error>> {
        let test = Test::<Sut>::new();
        let service_name = generate_service_name();
        let node = test.create_node();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let subscriber_name = PortName::new("brainslug").unwrap();
        let sut = service
            .subscriber_builder()
            .name(&subscriber_name)
            .create()?;

        assert_that!(sut.name(), eq subscriber_name);

        Ok(())
    }

    #[conformance_test]
    #[should_panic]
    #[cfg(debug_assertions)]
    pub fn subscriber_with_custom_payload_details_panics_when_calling_non_custom_receive<
        Sut: Service,
    >() {
        #[cfg(debug_assertions)]
        use iceoryx2::service::{
            builder::CustomPayloadMarker,
            static_config::message_type_details::{TypeDetail, TypeVariant},
        };

        const TYPE_SIZE_OVERRIDE: usize = 128;
        let test = Test::<Sut>::new();
        let node = test.create_node();
        let service_name = generate_service_name();
        let mut type_detail = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        type_detail_set_size(&mut type_detail, TYPE_SIZE_OVERRIDE);

        let service = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[CustomPayloadMarker]>()
                .__internal_set_payload_type_details(&type_detail)
                .create()
                .unwrap()
        };

        let sut = service.subscriber_builder().create().unwrap();

        // panics here
        let _sample = sut.receive();
    }
}
