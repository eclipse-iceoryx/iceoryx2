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

#[generic_tests::define]
mod subscriber {
    use iceoryx2::port::ReceiveError;
    use iceoryx2::service::builder::CustomPayloadMarker;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use std::collections::HashSet;

    use iceoryx2::{
        node::NodeBuilder,
        port::subscriber::SubscriberCreateError,
        service::{service_name::ServiceName, Service},
        testing,
    };
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "listener_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn receive_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", ReceiveError::ExceedsMaxBorrows), eq "ReceiveError::ExceedsMaxBorrows");
    }

    #[test]
    fn create_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", SubscriberCreateError::ExceedsMaxSupportedSubscribers), eq "SubscriberCreateError::ExceedsMaxSupportedSubscribers");
        assert_that!(
            format!("{}", SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService), eq "SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService");
    }

    #[test]
    fn id_is_unique<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_SUBSCRIBERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(MAX_SUBSCRIBERS)
            .create()
            .unwrap();

        let mut subscribers = vec![];
        let mut subscriber_id_set = HashSet::new();

        for _ in 0..MAX_SUBSCRIBERS {
            let subscriber = sut.subscriber_builder().create().unwrap();
            assert_that!(subscriber_id_set.insert(subscriber.id()), eq true);
            subscribers.push(subscriber);
        }
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn subscriber_with_custom_payload_details_panics_when_calling_non_custom_receive<
        Sut: Service,
    >() {
        const TYPE_SIZE_OVERRIDE: usize = 128;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_detail = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        testing::type_detail_set_size(&mut type_detail, TYPE_SIZE_OVERRIDE);

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

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    #[instantiate_tests(<iceoryx2::service::ipc_threadsafe::Service>)]
    mod ipc_threadsafe {}

    #[instantiate_tests(<iceoryx2::service::local_threadsafe::Service>)]
    mod local_threadsafe {}
}
