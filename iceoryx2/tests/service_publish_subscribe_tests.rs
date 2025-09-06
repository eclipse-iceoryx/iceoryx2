// Copyright (c) 2023 Contributors to the Eclipse Foundation
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
mod service_publish_subscribe {
    use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Barrier, Mutex};
    use std::thread;

    use iceoryx2::config::Config;
    use iceoryx2::port::publisher::PublisherCreateError;
    use iceoryx2::port::subscriber::SubscriberCreateError;
    use iceoryx2::port::update_connections::UpdateConnections;
    use iceoryx2::port::LoanError;
    use iceoryx2::prelude::{AllocationStrategy, *};
    use iceoryx2::service::builder::publish_subscribe::PublishSubscribeCreateError;
    use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenError;
    use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::service::{Service, ServiceDetails};
    use iceoryx2::testing;
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_elementary::alignment::Alignment;
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_log::{set_log_level, LogLevel};
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;

    #[derive(Debug, ZeroCopySend)]
    #[repr(C)]
    struct SomeUserHeader {
        value: [u64; 1024],
    }

    impl Default for SomeUserHeader {
        fn default() -> Self {
            Self {
                value: [0u64; 1024],
            }
        }
    }

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn open_or_create_with_attributes_succeeds_when_service_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr);

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_or_create_with_attributes_succeeds_when_attribute_is_satisfied<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new()
            .require(&"hello".try_into().unwrap(), &"world".try_into().unwrap());
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let attr1 = AttributeVerifier::new().require_key(&"hello".try_into().unwrap());
        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr1);

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_or_create_with_attributes_failed_when_service_payload_types_differ<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr);

        assert_that!(sut2, is_err);
    }

    #[test]
    fn open_or_create_with_attributes_failed_when_attribute_isnt_satisfied<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let attr = AttributeVerifier::new()
            .require(&"hello".try_into().unwrap(), &"world".try_into().unwrap());
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr);
        assert_that!(sut, is_ok);

        let attr1 = AttributeVerifier::new().require_key(&"non-exist".try_into().unwrap());
        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open_or_create_with_attributes(&attr1);

        assert_that!(sut2, is_err);
    }

    #[test]
    fn creating_non_existing_service_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();

        assert_that!(sut, is_ok);
        let sut = sut.unwrap();
        assert_that!(*sut.name(), eq service_name);
    }

    #[test]
    fn creating_same_service_twice_fails<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeCreateError::AlreadyExists
        );
    }

    #[test]
    fn recreate_after_drop_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq PublishSubscribeOpenError::DoesNotExist);
    }

    #[test]
    fn open_succeeds_when_service_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_has_wrong_type<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<i64>()
            .open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn open_fails_when_service_has_wrong_slice_base_type<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<[i64]>()
            .open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn open_fails_when_service_is_slice_based_and_typed_is_requested<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn open_fails_when_service_is_type_based_and_slice_is_requested<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .open();
        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_nodes_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_nodes(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_nodes(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_nodes(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_publishers_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(1)
            .open_or_create();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_max_subscribers_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_safe_overflow_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .enable_safe_overflow(false)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .enable_safe_overflow(true)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::IncompatibleOverflowBehavior
        );
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_history_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .history_size(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .history_size(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .history_size(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_subscriber_max_borrow_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_borrowed_samples(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_borrowed_samples(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_borrowed_samples(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_subscriber_max_buffer_size_requirement<
        Sut: Service,
    >() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_buffer_size(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_buffer_size(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_buffer_size(1)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_alignment_requirement<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .payload_alignment(Alignment::new(128).unwrap())
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .payload_alignment(Alignment::new(512).unwrap())
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            PublishSubscribeOpenError::IncompatibleTypes
        );

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .payload_alignment(Alignment::new(16).unwrap())
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_does_not_fail_when_service_owner_is_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut2, is_ok);

        drop(sut);

        let sut3 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut3, is_ok);
    }

    #[test]
    fn open_fails_when_all_previous_owners_have_been_dropped<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut2, is_ok);

        drop(sut);
        drop(sut2);

        let sut3 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq PublishSubscribeOpenError::DoesNotExist);
    }

    #[test]
    fn open_or_create_creates_service_if_it_does_not_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .open_or_create();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn open_or_create_opens_service_if_it_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let _sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open_or_create();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn max_publishers_and_subscribers_is_set_to_config_default<Sut: Service>() {
        let service_name = generate_name();
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let defaults = &Config::global_config().defaults;

        assert_that!(
            sut.static_config().max_publishers(), eq
            defaults.publish_subscribe.max_publishers
        );
        assert_that!(
            sut.static_config().max_subscribers(), eq
            defaults.publish_subscribe.max_subscribers
        );
    }

    #[test]
    fn open_uses_predefined_settings_when_nothing_is_specified<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_nodes(89)
            .max_publishers(4)
            .max_subscribers(5)
            .enable_safe_overflow(false)
            .history_size(6)
            .subscriber_max_borrowed_samples(7)
            .subscriber_max_buffer_size(8)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 89);
        assert_that!(sut.static_config().max_publishers(), eq 4);
        assert_that!(sut.static_config().max_subscribers(), eq 5);
        assert_that!(sut.static_config().has_safe_overflow(), eq false);
        assert_that!(sut.static_config().history_size(), eq 6);
        assert_that!(sut.static_config().subscriber_max_borrowed_samples(), eq 7);
        assert_that!(sut.static_config().subscriber_max_buffer_size(), eq 8);

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        assert_that!(sut2.static_config().max_nodes(), eq 89);
        assert_that!(sut2.static_config().max_publishers(), eq 4);
        assert_that!(sut2.static_config().max_subscribers(), eq 5);
        assert_that!(sut2.static_config().has_safe_overflow(), eq false);
        assert_that!(sut2.static_config().history_size(), eq 6);
        assert_that!(sut2.static_config().subscriber_max_borrowed_samples(), eq 7);
        assert_that!(sut2.static_config().subscriber_max_buffer_size(), eq 8);
    }

    #[test]
    fn settings_can_be_modified_via_custom_config<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let mut custom_config = config.clone();
        custom_config.defaults.publish_subscribe.max_nodes = 2;
        custom_config.defaults.publish_subscribe.max_publishers = 9;
        custom_config.defaults.publish_subscribe.max_subscribers = 10;
        custom_config
            .defaults
            .publish_subscribe
            .enable_safe_overflow = false;
        custom_config
            .defaults
            .publish_subscribe
            .publisher_history_size = 11;
        custom_config
            .defaults
            .publish_subscribe
            .subscriber_max_borrowed_samples = 12;
        custom_config
            .defaults
            .publish_subscribe
            .subscriber_max_buffer_size = 13;
        let node_1 = NodeBuilder::new()
            .config(&custom_config)
            .create::<Sut>()
            .unwrap();
        let node_2 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node_1
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 2);
        assert_that!(sut.static_config().max_publishers(), eq 9);
        assert_that!(sut.static_config().max_subscribers(), eq 10);
        assert_that!(sut.static_config().has_safe_overflow(), eq false);
        assert_that!(sut.static_config().history_size(), eq 11);
        assert_that!(sut.static_config().subscriber_max_borrowed_samples(), eq 12);
        assert_that!(sut.static_config().subscriber_max_buffer_size(), eq 13);

        let sut2 = node_2
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        // NOTE: although node_2 did specify a config with default values, since
        // node_1 was created first, the values of that node have to be preset
        assert_that!(sut2.static_config().max_nodes(), eq 2);
        assert_that!(sut2.static_config().max_publishers(), eq 9);
        assert_that!(sut2.static_config().max_subscribers(), eq 10);
        assert_that!(sut2.static_config().has_safe_overflow(), eq false);
        assert_that!(sut2.static_config().history_size(), eq 11);
        assert_that!(sut2.static_config().subscriber_max_borrowed_samples(), eq 12);
        assert_that!(sut2.static_config().subscriber_max_buffer_size(), eq 13);
    }

    #[test]
    fn number_of_publishers_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_PUBLISHERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(MAX_PUBLISHERS)
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        let mut publishers = vec![];

        for i in 0..MAX_PUBLISHERS / 2 {
            publishers.push(sut.publisher_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_publishers(), eq 2 * i + 1);
            assert_that!(sut2.dynamic_config().number_of_publishers(), eq 2 * i + 1);
            assert_that!(sut.dynamic_config().number_of_subscribers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_subscribers(), eq 0);

            publishers.push(sut2.publisher_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_publishers(), eq 2 * i + 2);
            assert_that!(sut2.dynamic_config().number_of_publishers(), eq 2 * i + 2);
            assert_that!(sut.dynamic_config().number_of_subscribers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_subscribers(), eq 0);
        }

        for i in 0..MAX_PUBLISHERS {
            publishers.pop();
            assert_that!(sut.dynamic_config().number_of_publishers(), eq MAX_PUBLISHERS - i - 1);
            assert_that!(sut2.dynamic_config().number_of_publishers(), eq MAX_PUBLISHERS - i - 1);
        }
    }

    #[test]
    fn type_informations_are_correct<Sut: Service>() {
        type Header = iceoryx2::service::header::publish_subscribe::Header;
        type PayloadType = u64;
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_name = generate_name();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<PayloadType>()
            .user_header::<SomeUserHeader>()
            .create()
            .unwrap();

        let d = sut.static_config().message_type_details();
        assert_that!(d.header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.header.type_name(), eq core::any::type_name::<Header>());
        assert_that!(d.header.size(), eq core::mem::size_of::<Header>());
        assert_that!(d.header.alignment(), eq core::mem::align_of::<Header>());
        assert_that!(d.user_header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.user_header.type_name(), eq core::any::type_name::<SomeUserHeader>());
        assert_that!(d.user_header.size(), eq core::mem::size_of::<SomeUserHeader>());
        assert_that!(d.user_header.alignment(), eq core::mem::align_of::<SomeUserHeader>());
        assert_that!(d.payload.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.payload.type_name(), eq core::any::type_name::<PayloadType>());
        assert_that!(d.payload.size(), eq core::mem::size_of::<PayloadType>());
        assert_that!(d.payload.alignment(), eq core::mem::align_of::<PayloadType>());
    }

    #[test]
    fn slice_type_informations_are_correct<Sut: Service>() {
        type Header = iceoryx2::service::header::publish_subscribe::Header;
        type PayloadType = u64;

        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<[PayloadType]>()
            .create()
            .unwrap();

        let d = sut.static_config().message_type_details();
        assert_that!(d.header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.header.type_name(), eq core::any::type_name::<Header>());
        assert_that!(d.header.size(), eq core::mem::size_of::<Header>());
        assert_that!(d.header.alignment(), eq core::mem::align_of::<Header>());
        assert_that!(d.user_header.variant(), eq TypeVariant::FixedSize);
        assert_that!(*d.user_header.type_name(), eq core::any::type_name::<()>());
        assert_that!(d.user_header.size(), eq core::mem::size_of::<()>());
        assert_that!(d.user_header.alignment(), eq core::mem::align_of::<()>());
        assert_that!(d.payload.variant(), eq TypeVariant::Dynamic);
        assert_that!(*d.payload.type_name(), eq core::any::type_name::<PayloadType>());
        assert_that!(d.payload.size(), eq core::mem::size_of::<PayloadType>());
        assert_that!(d.payload.alignment(), eq core::mem::align_of::<PayloadType>());
    }

    #[test]
    fn number_of_subscribers_works<Sut: Service>() {
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

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        let mut subscribers = vec![];

        for i in 0..MAX_SUBSCRIBERS / 2 {
            subscribers.push(sut.subscriber_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_subscribers(), eq 2 * i + 1);
            assert_that!(sut2.dynamic_config().number_of_subscribers(), eq 2 * i + 1);
            assert_that!(sut.dynamic_config().number_of_publishers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_publishers(), eq 0);

            subscribers.push(sut2.subscriber_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_subscribers(), eq 2 * i + 2);
            assert_that!(sut2.dynamic_config().number_of_subscribers(), eq 2 * i + 2);
            assert_that!(sut.dynamic_config().number_of_publishers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_publishers(), eq 0);
        }

        for i in 0..MAX_SUBSCRIBERS {
            subscribers.pop();
            assert_that!(sut.dynamic_config().number_of_subscribers(), eq MAX_SUBSCRIBERS - i - 1);
            assert_that!(sut2.dynamic_config().number_of_subscribers(), eq MAX_SUBSCRIBERS - i - 1);
        }
    }

    #[test]
    fn max_number_of_nodes_works<Sut: Service>() {
        let service_name = generate_name();
        const MAX_NODES: usize = 8;

        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_nodes(MAX_NODES)
            .create();
        assert_that!(sut, is_ok);

        let mut nodes = vec![];
        let mut services = vec![];

        nodes.push(node);
        services.push(sut.unwrap());

        for _ in 1..MAX_NODES {
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let sut = node
                .service_builder(&service_name)
                .publish_subscribe::<u64>()
                .open();
            assert_that!(sut, is_ok);

            nodes.push(node);
            services.push(sut.unwrap());
        }

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq PublishSubscribeOpenError::ExceedsMaxNumberOfNodes);

        nodes.pop();
        services.pop();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();

        assert_that!(sut, is_ok);
    }

    #[test]
    fn simple_communication_works_subscriber_created_first<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        let subscriber = sut.subscriber_builder().create().unwrap();
        let publisher = sut2.publisher_builder().create().unwrap();

        assert_that!(publisher.send_copy(1234), is_ok);
        assert_that!(publisher.send_copy(4567), is_ok);

        let result = subscriber.receive().unwrap();
        assert_that!(result, is_some);
        let sample = result.unwrap();
        assert_that!(*sample, eq 1234);
        assert_that!(*sample.payload(), eq 1234);

        let result = subscriber.receive().unwrap();
        assert_that!(result, is_some);
        let sample_2 = result.unwrap();
        assert_that!(*sample_2, eq 4567);
        assert_that!(*sample_2.payload(), eq 4567);
    }

    #[test]
    fn simple_communication_works_publisher_created_first<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut2.subscriber_builder().create().unwrap();
        assert_that!(publisher.update_connections(), is_ok);

        assert_that!(publisher.send_copy(1234), is_ok);
        assert_that!(publisher.send_copy(4567), is_ok);

        let result = subscriber.receive().unwrap();
        assert_that!(result, is_some);
        assert_that!(*result.unwrap(), eq 1234);

        let result = subscriber.receive().unwrap();
        assert_that!(result, is_some);
        assert_that!(*result.unwrap(), eq 4567);
    }

    #[test]
    fn custom_payload_alignment_cannot_be_smaller_than_payload_type_alignment<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .payload_alignment(Alignment::new(1).unwrap())
            .create()
            .unwrap();

        assert_that!(sut.static_config().message_type_details().payload.alignment(), eq core::mem::align_of::<u64>());
    }

    #[test]
    fn all_samples_are_correctly_aligned<Sut: Service>() {
        const BUFFER_SIZE: usize = 100;
        const ALIGNMENT: usize = 512;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_pub = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .subscriber_max_borrowed_samples(BUFFER_SIZE)
            .payload_alignment(Alignment::new(ALIGNMENT).unwrap())
            .create()
            .unwrap();

        let service_sub = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .open()
            .unwrap();

        let subscriber = service_sub.subscriber_builder().create().unwrap();
        let publisher = service_pub.publisher_builder().create().unwrap();

        let mut samples = vec![];
        for n in 0..BUFFER_SIZE {
            let mut sample = publisher.loan().unwrap();
            *sample.payload_mut() = n * 1920;

            let payload_address = (sample.payload() as *const usize) as usize;
            assert_that!(payload_address % ALIGNMENT, eq 0);
            sample.send().unwrap();

            let recv_sample = subscriber.receive().unwrap().unwrap();
            let recv_payload_address = (recv_sample.payload() as *const usize) as usize;
            assert_that!(recv_payload_address % ALIGNMENT, eq 0);
            assert_that!(*recv_sample, eq n * 1920);

            samples.push(recv_sample);
        }
    }

    #[test]
    fn publisher_reclaims_all_samples_after_disconnect<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const RECONNECTIONS: usize = 20;
        const MAX_SUBSCRIBERS: usize = 10;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(1)
            .max_subscribers(MAX_SUBSCRIBERS)
            .history_size(0)
            .subscriber_max_borrowed_samples(1)
            .subscriber_max_buffer_size(3)
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();

        for n in 0..MAX_SUBSCRIBERS {
            for _ in 0..RECONNECTIONS {
                let mut subscribers = vec![];
                for _ in 0..n {
                    subscribers.push(sut.subscriber_builder().create());
                }

                assert_that!(publisher.send_copy(1234), eq Ok(n));
                assert_that!(publisher.send_copy(4567), eq Ok(n));
                assert_that!(publisher.send_copy(789), eq Ok(n));
                subscribers.clear();
                assert_that!(publisher.send_copy(789), eq Ok(0));
                assert_that!(publisher.send_copy(789), eq Ok(0));
            }
        }
    }

    #[test]
    fn publisher_updates_connections_after_reconnect<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const RECONNECTIONS: usize = 20;
        const MAX_SUBSCRIBERS: usize = 10;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(1)
            .max_subscribers(MAX_SUBSCRIBERS)
            .history_size(0)
            .subscriber_max_borrowed_samples(1)
            .subscriber_max_buffer_size(3)
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();

        for n in 0..MAX_SUBSCRIBERS {
            for _ in 0..RECONNECTIONS {
                let mut subscribers = vec![];
                for _ in 0..n {
                    subscribers.push(sut.subscriber_builder().create().unwrap());
                }

                assert_that!(publisher.send_copy(1234), eq Ok(n));
                for subscriber in &subscribers {
                    assert_that!(subscriber.receive().unwrap(), is_some);
                }
            }
        }
    }

    #[test]
    fn subscriber_updates_connections_after_reconnect<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const RECONNECTIONS: usize = 20;
        const MAX_PUBLISHERS: usize = 10;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(MAX_PUBLISHERS)
            .history_size(0)
            .subscriber_max_borrowed_samples(1)
            .subscriber_max_buffer_size(1)
            .create()
            .unwrap();

        let subscriber = sut.subscriber_builder().create().unwrap();

        for n in 0..MAX_PUBLISHERS {
            for k in 0..RECONNECTIONS {
                let mut publishers = vec![];
                for _ in 0..n {
                    publishers.push(sut.publisher_builder().create().unwrap());
                }

                for publisher in publishers {
                    let payload: u64 = (n * k + 12) as _;
                    assert_that!(publisher.send_copy(payload), eq Ok(1));
                    let sample = subscriber.receive().unwrap();
                    assert_that!(sample, is_some);
                    assert_that!(*sample.unwrap(), eq payload);
                }
            }
        }
    }

    #[test]
    fn concurrent_communication_with_subscriber_reconnects_does_not_deadlock<Sut: Service>() {
        let _watch_dog = Watchdog::new();

        const NUMBER_OF_SUBSCRIBER_THREADS: usize = 2;
        const NUMBER_OF_RECONNECTIONS: usize = 50;

        let create_service_barrier = Barrier::new(2);
        let service_name = generate_name();
        let keep_running = AtomicBool::new(true);
        let config = testing::generate_isolated_config();
        let node = Mutex::new(NodeBuilder::new().config(&config).create::<Sut>().unwrap());

        thread::scope(|s| {
            s.spawn(|| {
                let sut2 = node
                    .lock()
                    .unwrap()
                    .service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .create()
                    .unwrap();
                let publisher = sut2.publisher_builder().create().unwrap();

                create_service_barrier.wait();
                let mut counter = 1u64;

                while keep_running.load(Ordering::Relaxed) {
                    assert_that!(publisher.send_copy(counter), is_ok);
                    counter += 1;
                }
            });

            create_service_barrier.wait();
            let mut threads = vec![];
            for _ in 0..NUMBER_OF_SUBSCRIBER_THREADS {
                threads.push(s.spawn(|| {
                    let sut = node
                        .lock()
                        .unwrap()
                        .service_builder(&service_name)
                        .publish_subscribe::<u64>()
                        .open()
                        .unwrap();

                    let mut latest_counter = 0u64;
                    for _ in 0..NUMBER_OF_RECONNECTIONS {
                        let subscriber = sut.subscriber_builder().create().unwrap();

                        loop {
                            if let Some(counter) = subscriber.receive().unwrap() {
                                assert_that!(latest_counter, lt * counter);
                                latest_counter = *counter;
                                break;
                            }
                        }
                    }
                }));
            }

            for t in threads {
                t.join().unwrap();
            }
            keep_running.store(false, Ordering::Relaxed);
        });
    }

    #[test]
    fn concurrent_communication_with_publisher_reconnects_does_not_deadlock<Sut: Service>() {
        let _watch_dog = Watchdog::new();

        const NUMBER_OF_PUBLISHER_THREADS: usize = 2;
        const NUMBER_OF_RECONNECTIONS: usize = 50;

        let create_service_barrier = Barrier::new(1 + NUMBER_OF_PUBLISHER_THREADS);
        let service_name = generate_name();
        let keep_running = AtomicBool::new(true);
        let reconnection_cycle = AtomicUsize::new(0);
        let config = testing::generate_isolated_config();
        let node = Mutex::new(NodeBuilder::new().config(&config).create::<Sut>().unwrap());

        thread::scope(|s| {
            s.spawn(|| {
                let sut = node
                    .lock()
                    .unwrap()
                    .service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .max_publishers(NUMBER_OF_PUBLISHER_THREADS)
                    .open_or_create()
                    .unwrap();

                let subscriber = sut.subscriber_builder().create().unwrap();
                create_service_barrier.wait();

                while keep_running.load(Ordering::Relaxed) {
                    if let Some(_) = subscriber.receive().unwrap() {
                        if reconnection_cycle.fetch_add(1, Ordering::Relaxed)
                            == NUMBER_OF_RECONNECTIONS
                        {
                            keep_running.store(false, Ordering::Relaxed);
                        }
                    }
                }
            });

            for _ in 0..NUMBER_OF_PUBLISHER_THREADS {
                s.spawn(|| {
                    let sut2 = node
                        .lock()
                        .unwrap()
                        .service_builder(&service_name)
                        .publish_subscribe::<u64>()
                        .max_publishers(NUMBER_OF_PUBLISHER_THREADS)
                        .open_or_create()
                        .unwrap();

                    create_service_barrier.wait();

                    while keep_running.load(Ordering::Relaxed) {
                        let publisher = sut2.publisher_builder().create().unwrap();

                        let current_cycle = reconnection_cycle.load(Ordering::Relaxed);
                        let mut counter = 1u64;
                        while current_cycle == reconnection_cycle.load(Ordering::Relaxed)
                            && keep_running.load(Ordering::Relaxed)
                        {
                            assert_that!(publisher.send_copy(counter), is_ok);
                            counter += 1;
                        }
                    }
                });
            }
        });
    }

    #[test]
    fn communication_with_max_subscribers_and_publishers<Sut: Service>() {
        const MAX_PUB: usize = 4;
        const MAX_SUB: usize = 6;
        const NUMBER_OF_ITERATIONS: u64 = 128;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(MAX_PUB)
            .max_subscribers(MAX_SUB)
            .create()
            .unwrap();

        let mut publishers = vec![];
        publishers.reserve(MAX_PUB);

        for _ in 0..MAX_PUB {
            publishers.push(sut.publisher_builder().create().unwrap());
        }

        let mut subscribers = vec![];
        subscribers.reserve(MAX_SUB);

        for _ in 0..MAX_SUB {
            subscribers.push(sut.subscriber_builder().create().unwrap());
        }

        let mut counter = 0;
        for _ in 0..NUMBER_OF_ITERATIONS {
            for publisher in &mut publishers {
                assert_that!(publisher.send_copy(counter), is_ok);

                for subscriber in &subscribers {
                    let sample = subscriber.receive().unwrap();
                    assert_that!(sample, is_some);
                    assert_that!(*sample.unwrap(), eq counter);
                }
                counter += 1;
            }
        }
    }

    #[test]
    fn multi_channel_communication_with_max_subscribers_and_publishers<Sut: Service>() {
        const MAX_PUB: usize = 5;
        const MAX_SUB: usize = 7;
        const NUMBER_OF_ITERATIONS: u64 = 128;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(MAX_PUB)
            .max_subscribers(MAX_SUB)
            .create()
            .unwrap();

        let mut channels = vec![];
        channels.reserve(MAX_PUB + MAX_SUB);

        for _ in 0..MAX_PUB + MAX_SUB {
            channels.push(
                node.service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .open()
                    .unwrap(),
            );
        }

        let mut publishers = vec![];
        publishers.reserve(MAX_PUB);

        for c in channels.iter().take(MAX_PUB) {
            publishers.push(c.publisher_builder().create().unwrap());
        }

        let mut subscribers = vec![];
        subscribers.reserve(MAX_SUB);

        for i in 0..MAX_SUB {
            subscribers.push(channels[i + MAX_PUB].subscriber_builder().create().unwrap());
        }

        let mut counter = 0;
        for _ in 0..NUMBER_OF_ITERATIONS {
            for publisher in &mut publishers {
                assert_that!(publisher.send_copy(counter), is_ok);

                for subscriber in &subscribers {
                    let sample = subscriber.receive().unwrap();
                    assert_that!(sample, is_some);
                    assert_that!(*sample.unwrap(), eq counter);
                }
                counter += 1;
            }
        }
    }

    #[test]
    fn publish_safely_overflows_when_enabled<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const BUFFER_SIZE: usize = 2;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .enable_safe_overflow(true)
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        for i in 0..BUFFER_SIZE {
            assert_that!(publisher.send_copy(i), is_ok);
        }

        for i in 0..BUFFER_SIZE {
            assert_that!(publisher.send_copy(2 * i + 25), is_ok);
        }

        for i in 0..BUFFER_SIZE {
            let sample = subscriber.receive().unwrap().unwrap();
            assert_that!(*sample, eq 2 * i + 25);
        }
    }

    #[test]
    fn publish_does_not_overflow_when_deactivated<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const BUFFER_SIZE: usize = 5;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .enable_safe_overflow(false)
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .create()
            .unwrap();

        let publisher = sut
            .publisher_builder()
            .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
            .create()
            .unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        for i in 0..BUFFER_SIZE {
            assert_that!(publisher.send_copy(i), is_ok);
        }

        for i in 0..BUFFER_SIZE {
            assert_that!(publisher.send_copy(2 * i + 25), is_ok);
        }

        for i in 0..BUFFER_SIZE {
            let sample = subscriber.receive().unwrap().unwrap();
            assert_that!(*sample, eq i);
        }
    }

    #[test]
    fn publish_non_overflow_with_greater_history_than_buffer_fails<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .enable_safe_overflow(false)
            .history_size(12)
            .subscriber_max_buffer_size(11)
            .create();

        assert_that!(sut, is_err);
        assert_that!(
            sut.err().unwrap(), eq
            PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize
        );
    }

    #[test]
    fn publish_history_is_delivered_on_subscription<Sut: Service>() {
        const BUFFER_SIZE: usize = 2;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .history_size(3)
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .create()
            .unwrap();

        let sut_publisher = sut.publisher_builder().create().unwrap();
        assert_that!(sut_publisher.send_copy(29), is_ok);
        assert_that!(sut_publisher.send_copy(32), is_ok);
        assert_that!(sut_publisher.send_copy(35), is_ok);

        let sut_subscriber = sut.subscriber_builder().create().unwrap();
        assert_that!(sut_publisher.update_connections(), is_ok);

        for i in 0..BUFFER_SIZE {
            let data = sut_subscriber.receive().unwrap();
            assert_that!(data, is_some);
            assert_that!(*data.unwrap(), eq 29 + (i + 1) * 3 )
        }
    }

    #[test]
    fn publish_history_of_zero_works<Sut: Service>() {
        const BUFFER_SIZE: usize = 2;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .history_size(0)
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .create()
            .unwrap();

        let sut_publisher = sut.publisher_builder().create().unwrap();
        assert_that!(sut_publisher.send_copy(29), is_ok);

        let sut_subscriber = sut.subscriber_builder().create().unwrap();
        assert_that!(sut_publisher.update_connections(), is_ok);

        let data = sut_subscriber.receive().unwrap();
        assert_that!(data, is_none);
    }

    #[test]
    fn publish_send_copy_with_huge_overflow_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const BUFFER_SIZE: usize = 5;

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .max_publishers(1)
            .max_subscribers(2)
            .history_size(0)
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .subscriber_max_borrowed_samples(1)
            .create()
            .unwrap();

        let sut_publisher = sut
            .publisher_builder()
            .max_loaned_samples(1)
            .create()
            .unwrap();

        let mut subscribers = vec![];
        for _ in 0..2 {
            let sut_subscriber = sut.subscriber_builder().create();
            subscribers.push(sut_subscriber);
        }

        for _ in 0..BUFFER_SIZE * 100 {
            assert_that!(sut_publisher.send_copy(889), is_ok);
        }
    }

    fn publisher_never_goes_out_of_memory_impl<Sut: Service>(
        buffer_size: usize,
        history_size: usize,
        max_borrow: usize,
        max_subscribers: usize,
        max_loan: usize,
    ) {
        const ITERATIONS: usize = 16;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .max_publishers(1)
            .max_subscribers(max_subscribers)
            .enable_safe_overflow(true)
            .history_size(history_size)
            .subscriber_max_buffer_size(buffer_size)
            .subscriber_max_borrowed_samples(max_borrow)
            .create()
            .unwrap();

        let sut_publisher = sut
            .publisher_builder()
            .max_loaned_samples(max_loan)
            .create()
            .unwrap();

        let mut subscribers = vec![];
        for _ in 0..max_subscribers {
            let sut_subscriber = sut.subscriber_builder().create().unwrap();
            subscribers.push(sut_subscriber);
        }

        for _ in 0..ITERATIONS {
            // max out borrow
            let mut borrowed_samples = vec![];
            while borrowed_samples.len() < max_borrow * max_subscribers {
                for _ in 0..buffer_size {
                    assert_that!(sut_publisher.send_copy(889), is_ok);
                }

                for (i, s) in subscribers.iter().enumerate().take(max_subscribers) {
                    while let Ok(Some(sample)) = s.receive() {
                        borrowed_samples.push((i, sample));
                    }
                }
            }

            // max out buffer
            for _ in 0..buffer_size {
                assert_that!(sut_publisher.send_copy(8127), is_ok);
            }

            // max out history
            for _ in 0..history_size {
                assert_that!(sut_publisher.send_copy(17283), is_ok);
            }

            // max out loan
            let mut loaned_samples = vec![];
            for _ in 0..max_loan {
                let sample = sut_publisher.loan_uninit();
                assert_that!(sample, is_ok);
                loaned_samples.push(sample);
            }

            let sample = sut_publisher.loan_uninit();
            assert_that!(sample, is_err);
            assert_that!(sample.err().unwrap(), eq LoanError::ExceedsMaxLoans);

            // cleanup
            borrowed_samples.clear();
            loaned_samples.clear();
            for s in subscribers.iter().take(max_subscribers) {
                while let Ok(Some(_)) = s.receive() {}
            }
        }
    }

    #[test]
    fn publisher_never_goes_out_of_memory_with_huge_max_loan<Sut: Service>() {
        const BUFFER_SIZE: usize = 1;
        const HISTORY_SIZE: usize = 0;
        const MAX_BORROW: usize = 1;
        const MAX_SUBSCRIBERS: usize = 1;
        const MAX_LOAN: usize = 100;

        publisher_never_goes_out_of_memory_impl::<Sut>(
            BUFFER_SIZE,
            HISTORY_SIZE,
            MAX_BORROW,
            MAX_SUBSCRIBERS,
            MAX_LOAN,
        );
    }

    #[test]
    fn publisher_never_goes_out_of_memory_with_huge_max_subscribers<Sut: Service>() {
        const BUFFER_SIZE: usize = 1;
        const HISTORY_SIZE: usize = 0;
        const MAX_BORROW: usize = 1;
        const MAX_SUBSCRIBERS: usize = 50;
        const MAX_LOAN: usize = 1;

        publisher_never_goes_out_of_memory_impl::<Sut>(
            BUFFER_SIZE,
            HISTORY_SIZE,
            MAX_BORROW,
            MAX_SUBSCRIBERS,
            MAX_LOAN,
        );
    }

    #[test]
    fn publisher_never_goes_out_of_memory_with_huge_max_borrow<Sut: Service>() {
        const BUFFER_SIZE: usize = 1;
        const HISTORY_SIZE: usize = 0;
        const MAX_BORROW: usize = 100;
        const MAX_SUBSCRIBERS: usize = 1;
        const MAX_LOAN: usize = 1;

        publisher_never_goes_out_of_memory_impl::<Sut>(
            BUFFER_SIZE,
            HISTORY_SIZE,
            MAX_BORROW,
            MAX_SUBSCRIBERS,
            MAX_LOAN,
        );
    }

    #[test]
    fn publisher_never_goes_out_of_memory_with_huge_history_size<Sut: Service>() {
        const BUFFER_SIZE: usize = 1;
        const HISTORY_SIZE: usize = 100;
        const MAX_BORROW: usize = 1;
        const MAX_SUBSCRIBERS: usize = 1;
        const MAX_LOAN: usize = 1;

        publisher_never_goes_out_of_memory_impl::<Sut>(
            BUFFER_SIZE,
            HISTORY_SIZE,
            MAX_BORROW,
            MAX_SUBSCRIBERS,
            MAX_LOAN,
        );
    }

    #[test]
    fn publisher_never_goes_out_of_memory_with_huge_buffer_size<Sut: Service>() {
        const BUFFER_SIZE: usize = 3;
        const HISTORY_SIZE: usize = 0;
        const MAX_BORROW: usize = 1;
        const MAX_SUBSCRIBERS: usize = 1;
        const MAX_LOAN: usize = 1;

        publisher_never_goes_out_of_memory_impl::<Sut>(
            BUFFER_SIZE,
            HISTORY_SIZE,
            MAX_BORROW,
            MAX_SUBSCRIBERS,
            MAX_LOAN,
        );
    }

    #[test]
    fn publisher_never_goes_out_of_memory_with_huge_values<Sut: Service>() {
        const BUFFER_SIZE: usize = 29;
        const HISTORY_SIZE: usize = 31;
        const MAX_BORROW: usize = 12;
        const MAX_SUBSCRIBERS: usize = 25;
        const MAX_LOAN: usize = 35;

        publisher_never_goes_out_of_memory_impl::<Sut>(
            BUFFER_SIZE,
            HISTORY_SIZE,
            MAX_BORROW,
            MAX_SUBSCRIBERS,
            MAX_LOAN,
        );
    }

    fn retrieve_channel_capacity_is_never_exceeded<Sut: Service>(
        publisher_borrow_size: usize,
        buffer_size: usize,
        max_borrow: usize,
    ) {
        const ITERATIONS: usize = 16;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .max_publishers(1)
            .max_subscribers(1)
            .enable_safe_overflow(false)
            .history_size(0)
            .subscriber_max_buffer_size(buffer_size)
            .subscriber_max_borrowed_samples(max_borrow)
            .create()
            .unwrap();

        let sut_publisher = sut
            .publisher_builder()
            .max_loaned_samples(publisher_borrow_size)
            .create()
            .unwrap();
        let sut_subscriber = sut.subscriber_builder().create().unwrap();

        let mut borrowed_samples = vec![];
        let mut cached_samples = vec![];

        let mut send_sample = || {
            if borrowed_samples.is_empty() {
                for _ in 0..publisher_borrow_size {
                    borrowed_samples.push(sut_publisher.loan().unwrap());
                }
            }

            let sample = borrowed_samples.pop().unwrap();
            sample.send().unwrap();
        };

        for _ in 0..ITERATIONS {
            for _ in 0..max_borrow {
                send_sample();
                cached_samples.push(sut_subscriber.receive().unwrap().unwrap());
            }

            for _ in 0..buffer_size {
                send_sample();
            }

            cached_samples.clear();
            for _ in 0..buffer_size {
                sut_subscriber.receive().unwrap().unwrap();
            }
        }
    }

    #[test]
    fn retrieve_channel_capacity_is_never_exceeded_with_large_publisher_borrow_size<
        Sut: Service,
    >() {
        const PUBLISHER_BORROW_SIZE: usize = 10;
        const BUFFER_SIZE: usize = 1;
        const MAX_BORROW: usize = 1;
        retrieve_channel_capacity_is_never_exceeded::<Sut>(
            PUBLISHER_BORROW_SIZE,
            BUFFER_SIZE,
            MAX_BORROW,
        );
    }

    #[test]
    fn retrieve_channel_capacity_is_never_exceeded_with_large_buffer_size<Sut: Service>() {
        const PUBLISHER_BORROW_SIZE: usize = 1;
        const BUFFER_SIZE: usize = 10;
        const MAX_BORROW: usize = 1;
        retrieve_channel_capacity_is_never_exceeded::<Sut>(
            PUBLISHER_BORROW_SIZE,
            BUFFER_SIZE,
            MAX_BORROW,
        );
    }

    #[test]
    fn retrieve_channel_capacity_is_never_exceeded_with_large_max_borrow<Sut: Service>() {
        const PUBLISHER_BORROW_SIZE: usize = 1;
        const BUFFER_SIZE: usize = 1;
        const MAX_BORROW: usize = 10;

        retrieve_channel_capacity_is_never_exceeded::<Sut>(
            PUBLISHER_BORROW_SIZE,
            BUFFER_SIZE,
            MAX_BORROW,
        );
    }

    #[test]
    fn retrieve_channel_capacity_is_never_exceeded_with_large_settings<Sut: Service>() {
        const PUBLISHER_BORROW_SIZE: usize = 20;
        const BUFFER_SIZE: usize = 14;
        const MAX_BORROW: usize = 15;

        retrieve_channel_capacity_is_never_exceeded::<Sut>(
            PUBLISHER_BORROW_SIZE,
            BUFFER_SIZE,
            MAX_BORROW,
        );
    }

    #[test]
    fn retrieve_channel_capacity_is_never_exceeded_with_small_settings<Sut: Service>() {
        const PUBLISHER_BORROW_SIZE: usize = 1;
        const BUFFER_SIZE: usize = 1;
        const MAX_BORROW: usize = 1;

        retrieve_channel_capacity_is_never_exceeded::<Sut>(
            PUBLISHER_BORROW_SIZE,
            BUFFER_SIZE,
            MAX_BORROW,
        );
    }

    #[test]
    fn creating_max_supported_amount_of_ports_work<Sut: Service>() {
        const MAX_PUBLISHERS: usize = 4;
        const MAX_SUBSCRIBERS: usize = 8;

        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(MAX_PUBLISHERS)
            .max_subscribers(MAX_SUBSCRIBERS)
            .create()
            .unwrap();

        let mut publishers = vec![];
        let mut subscribers = vec![];

        // acquire all possible ports
        for _ in 0..MAX_PUBLISHERS {
            let publisher = sut.publisher_builder().create();
            assert_that!(publisher, is_ok);
            publishers.push(publisher);
        }

        for _ in 0..MAX_SUBSCRIBERS {
            let subscriber = sut.subscriber_builder().create();
            assert_that!(subscriber, is_ok);
            subscribers.push(subscriber);
        }

        // create additional ports and fail
        let publisher = sut.publisher_builder().create();
        assert_that!(publisher, is_err);
        assert_that!(
            publisher.err().unwrap(), eq
            PublisherCreateError::ExceedsMaxSupportedPublishers
        );

        let subscriber = sut.subscriber_builder().create();
        assert_that!(subscriber, is_err);
        assert_that!(
            subscriber.err().unwrap(), eq
            SubscriberCreateError::ExceedsMaxSupportedSubscribers
        );

        // remove a publisher and subscriber
        assert_that!(publishers.remove(0), is_ok);
        assert_that!(subscribers.remove(0), is_ok);

        // create additional ports shall work again
        let publisher = sut.publisher_builder().create();
        assert_that!(publisher, is_ok);

        let subscriber = sut.subscriber_builder().create();
        assert_that!(subscriber, is_ok);
    }

    #[test]
    fn set_max_nodes_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_nodes(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 1);
    }

    #[test]
    fn set_max_publishers_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_publishers(), eq 1);
    }

    #[test]
    fn set_max_subscribers_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_subscribers(), eq 1);
    }

    #[test]
    fn set_subscriber_max_borrowed_samples_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_borrowed_samples(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().subscriber_max_borrowed_samples(), eq 1);
    }

    #[test]
    fn set_buffer_size_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_buffer_size(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().subscriber_max_buffer_size(), eq 1);
    }

    #[test]
    fn does_exist_works_single<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::PublishSubscribe).unwrap(), eq false);

        let _sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::PublishSubscribe).unwrap(), eq true);
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::PublishSubscribe).unwrap(), eq true);

        drop(_sut);

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::PublishSubscribe).unwrap(), eq false);
    }

    #[test]
    fn does_exist_works_many<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 8;

        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        for i in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();
            assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::PublishSubscribe).unwrap(), eq false);

            services.push(
                node.service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .create()
                    .unwrap(),
            );
            service_names.push(service_name);

            for s in service_names.iter().take(i + 1) {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::PublishSubscribe).unwrap(), eq true);
            }
        }

        for i in 0..NUMBER_OF_SERVICES {
            for s in service_names.iter().take(NUMBER_OF_SERVICES - i) {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::PublishSubscribe).unwrap(), eq true);
            }

            for s in service_names
                .iter()
                .take(NUMBER_OF_SERVICES)
                .skip(NUMBER_OF_SERVICES - i)
            {
                assert_that!(Sut::does_exist(s, &config, MessagingPattern::PublishSubscribe).unwrap(), eq false);
            }

            services.pop();
        }
    }

    #[test]
    fn list_works<Sut: Service>() {
        const NUMBER_OF_SERVICES: usize = 8;

        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut services = vec![];
        let mut service_names = vec![];

        let contains_service_names = |names, state: Vec<ServiceDetails<Sut>>| {
            for n in names {
                let mut name_found = false;
                for s in &state {
                    if *s.static_details.name() == n {
                        name_found = true;
                        break;
                    }
                }

                if !name_found {
                    return false;
                }
            }

            true
        };

        for i in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();

            services.push(
                node.service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .create()
                    .unwrap(),
            );
            service_names.push(service_name);

            let mut service_list = vec![];
            Sut::list(&config, |s| {
                service_list.push(s);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(service_list, len i + 1);

            assert_that!(contains_service_names(service_names.clone(), service_list), eq true);
        }

        for i in 0..NUMBER_OF_SERVICES {
            services.pop();
            service_names.pop();

            let mut service_list = vec![];
            Sut::list(&config, |s| {
                service_list.push(s);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(service_list, len NUMBER_OF_SERVICES - i - 1);
            assert_that!(contains_service_names(service_names.clone(), service_list), eq true);
        }
    }

    #[test]
    fn dropping_service_keeps_established_communication<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        drop(sut);

        const PAYLOAD: u64 = 98129312938;

        assert_that!(publisher.send_copy(PAYLOAD), eq Ok(1));
        assert_that!(*subscriber.receive().unwrap().unwrap(), eq PAYLOAD);
    }

    #[test]
    fn ports_of_dropped_service_block_new_service_creation<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let subscriber = sut.subscriber_builder().create().unwrap();
        let publisher = sut.publisher_builder().create().unwrap();

        drop(sut);

        assert_that!(node.service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create().err().unwrap(),
            eq PublishSubscribeCreateError::AlreadyExists);

        drop(subscriber);

        assert_that!(node.service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create().err().unwrap(),
            eq PublishSubscribeCreateError::AlreadyExists);

        drop(publisher);

        assert_that!(
            node.service_builder(&service_name)
                .publish_subscribe::<u64>()
                .create(),
            is_ok
        );
    }

    #[test]
    fn service_can_be_opened_when_there_is_a_publisher<Sut: Service>() {
        let payload = 1809723987;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();
        let publisher = sut.publisher_builder().create().unwrap();

        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut, is_ok);
        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut.err().unwrap(), eq PublishSubscribeCreateError::AlreadyExists);
        drop(subscriber);

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();
        publisher.send_copy(payload).unwrap();
        let sample = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample.payload(), eq payload);

        drop(subscriber);
        drop(sut);
        drop(publisher);
        drop(sample);

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut.err().unwrap(), eq PublishSubscribeOpenError::DoesNotExist);
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn service_can_be_opened_when_there_is_a_subscriber<Sut: Service>() {
        let payload = 59123544;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();
        let publisher = sut.publisher_builder().create().unwrap();

        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut, is_ok);
        drop(sut);
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut.err().unwrap(), eq PublishSubscribeCreateError::AlreadyExists);
        drop(publisher);

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();
        let publisher = sut.publisher_builder().create().unwrap();
        publisher.send_copy(payload).unwrap();
        let sample = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample.payload(), eq payload);

        drop(publisher);
        drop(sut);
        drop(subscriber);
        drop(sample);

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();
        assert_that!(sut.err().unwrap(), eq PublishSubscribeOpenError::DoesNotExist);
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn subscriber_can_decrease_buffer_size<Sut: Service>() {
        const BUFFER_SIZE: usize = 16;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .open()
            .unwrap();

        for i in 1..=BUFFER_SIZE {
            let publisher_before_sub = sut2.publisher_builder().create().unwrap();
            let subscriber = sut.subscriber_builder().buffer_size(i).create().unwrap();
            let publisher_after_sub = sut2.publisher_builder().create().unwrap();

            assert_that!(subscriber.buffer_size(), eq i);

            for n in 0..i * 2 {
                assert_that!(publisher_before_sub.send_copy(n), is_ok);
            }

            for n in 0..i {
                let sample = subscriber.receive().unwrap().unwrap();
                assert_that!(*sample, eq i + n);
            }

            let sample = subscriber.receive().unwrap();
            assert_that!(sample, is_none);

            for n in 0..i * 2 {
                assert_that!(publisher_after_sub.send_copy(n as _), is_ok);
            }

            for n in 0..i {
                let sample = subscriber.receive().unwrap().unwrap();
                assert_that!(*sample, eq i + n);
            }

            let sample = subscriber.receive().unwrap();
            assert_that!(sample, is_none);
        }
    }

    #[test]
    fn subscriber_creation_fails_when_buffer_size_exceeds_service_max<Sut: Service>() {
        const BUFFER_SIZE: usize = 16;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_buffer_size(BUFFER_SIZE)
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .unwrap();

        let subscriber = sut2
            .subscriber_builder()
            .buffer_size(BUFFER_SIZE + 1)
            .create();
        assert_that!(subscriber, is_err);
        assert_that!(subscriber.err().unwrap(), eq SubscriberCreateError::BufferSizeExceedsMaxSupportedBufferSizeOfService);
    }

    #[test]
    fn subscriber_buffer_size_is_at_least_one<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let subscriber = sut.subscriber_builder().buffer_size(0).create().unwrap();
        assert_that!(subscriber.buffer_size(), eq 1);
    }

    #[test]
    fn sliced_service_works<Sut: Service>() {
        const MAX_ELEMENTS: usize = 91;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .create()
            .unwrap();

        let publisher = sut
            .publisher_builder()
            .initial_max_slice_len(MAX_ELEMENTS)
            .create()
            .unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        for n in 0..=MAX_ELEMENTS {
            let sample = publisher.loan_slice_uninit(n).unwrap();
            sample.write_from_fn(|i| i as u64 * 25).send().unwrap();

            let recv_sample = subscriber.receive().unwrap().unwrap();

            assert_that!(recv_sample.payload(), len n);
            for (i, element) in recv_sample.payload().iter().enumerate() {
                assert_that!(*element, eq i as u64 * 25);
            }
        }
    }

    #[test]
    fn slice_aligned_service_works<Sut: Service>() {
        const MAX_ELEMENTS: usize = 91;
        const ALIGNMENT: usize = 64;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_pub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .subscriber_max_buffer_size(MAX_ELEMENTS + 1)
            .subscriber_max_borrowed_samples(MAX_ELEMENTS + 1)
            .payload_alignment(Alignment::new(ALIGNMENT).unwrap())
            .create()
            .unwrap();

        let service_sub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u64]>()
            .open()
            .unwrap();

        let publisher = service_pub
            .publisher_builder()
            .initial_max_slice_len(MAX_ELEMENTS)
            .create()
            .unwrap();
        let subscriber = service_sub.subscriber_builder().create().unwrap();

        let mut samples = vec![];
        for n in 0..=MAX_ELEMENTS {
            let sample = publisher.loan_slice_uninit(n).unwrap();
            assert_that!((sample.payload().as_ptr() as usize) % ALIGNMENT, eq 0);
            sample.write_from_fn(|i| i as u64 * 25).send().unwrap();

            let recv_sample = subscriber.receive().unwrap().unwrap();

            assert_that!((recv_sample.payload().as_ptr() as usize) % ALIGNMENT, eq 0);
            assert_that!(recv_sample.payload(), len n);
            for (i, element) in recv_sample.payload().iter().enumerate() {
                assert_that!(*element, eq i as u64 * 25);
            }
            samples.push(recv_sample);
        }
    }

    #[test]
    fn simple_communication_with_user_header_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .user_header::<SomeUserHeader>()
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .user_header::<SomeUserHeader>()
            .open()
            .unwrap();

        let subscriber = sut.subscriber_builder().create().unwrap();
        let publisher = sut2.publisher_builder().create().unwrap();
        let mut sample = publisher.loan().unwrap();

        for i in 0..1024 {
            sample.user_header_mut().value[i] = i as u64;
        }
        *sample.payload_mut() = 1829731;
        sample.send().unwrap();

        let result = subscriber.receive().unwrap();
        assert_that!(result, is_some);
        let sample = result.unwrap();
        assert_that!(*sample, eq 1829731);
        assert_that!(*sample.payload(), eq 1829731);

        for i in 0..1024 {
            assert_that!(sample.user_header().value[i], eq i as u64);
        }
    }

    #[test]
    fn same_payload_type_but_different_user_header_does_not_connect<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .user_header::<SomeUserHeader>()
            .create()
            .unwrap();

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();

        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn create_with_custom_payload_type_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[CustomPayloadMarker]>()
                .__internal_set_payload_type_details(&TypeDetail::new::<u64>(
                    TypeVariant::FixedSize,
                ))
                .create()
                .unwrap()
        };

        let sut2 = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open();

        assert_that!(sut2, is_ok);

        let sut3 = node
            .service_builder(&service_name)
            .publish_subscribe::<u32>()
            .open();

        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn create_with_custom_user_header_type_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const HEADER_SIZE: usize = 1024;

        let sut_pub = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[u8]>()
                .user_header::<CustomHeaderMarker>()
                .__internal_set_user_header_type_details(&TypeDetail::new::<[u64; HEADER_SIZE]>(
                    TypeVariant::FixedSize,
                ))
                .create()
                .unwrap()
        };

        let sut_sub = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[u8]>()
                .user_header::<CustomHeaderMarker>()
                .__internal_set_user_header_type_details(&TypeDetail::new::<[u64; HEADER_SIZE]>(
                    TypeVariant::FixedSize,
                ))
                .open()
                .unwrap()
        };

        let sut3 = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open();

        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);

        let publisher = sut_pub.publisher_builder().create().unwrap();
        let subscriber = sut_sub.subscriber_builder().create().unwrap();

        let mut sample = publisher.loan_slice(1).unwrap();
        let header = (sample.user_header_mut() as *mut CustomHeaderMarker) as *mut u64;
        for i in 0..HEADER_SIZE {
            unsafe { *header.add(i) = (4 * i + 1) as u64 };
        }
        sample.send().unwrap();

        let sample = subscriber.receive().unwrap().unwrap();
        let header = (sample.user_header() as *const CustomHeaderMarker) as *const u64;

        for i in 0..HEADER_SIZE {
            assert_that!(unsafe { *header.add(i) }, eq(4 * i + 1) as u64);
        }
    }

    #[test]
    fn open_with_custom_payload_type_works<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u128>()
            .create()
            .unwrap();

        let sut2 = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[CustomPayloadMarker]>()
                .__internal_set_payload_type_details(&TypeDetail::new::<u128>(
                    TypeVariant::FixedSize,
                ))
                .open()
        };

        assert_that!(sut2, is_ok);

        let sut3 = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[CustomPayloadMarker]>()
                .__internal_set_payload_type_details(&TypeDetail::new::<u64>(
                    TypeVariant::FixedSize,
                ))
                .open()
        };

        assert_that!(sut3, is_err);
        assert_that!(sut3.err().unwrap(), eq PublishSubscribeOpenError::IncompatibleTypes);
    }

    #[test]
    fn open_error_display_works<Sut: Service>() {
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotExist), eq
                                  "PublishSubscribeOpenError::DoesNotExist");
        assert_that!(format!("{}", PublishSubscribeOpenError::InternalFailure), eq
                                  "PublishSubscribeOpenError::InternalFailure");
        assert_that!(format!("{}", PublishSubscribeOpenError::IncompatibleTypes), eq
                                  "PublishSubscribeOpenError::IncompatibleTypes");
        assert_that!(format!("{}", PublishSubscribeOpenError::IncompatibleMessagingPattern), eq
                                  "PublishSubscribeOpenError::IncompatibleMessagingPattern");
        assert_that!(format!("{}", PublishSubscribeOpenError::IncompatibleAttributes), eq
                                  "PublishSubscribeOpenError::IncompatibleAttributes");
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize), eq
                                  "PublishSubscribeOpenError::DoesNotSupportRequestedMinBufferSize");
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize), eq
                                  "PublishSubscribeOpenError::DoesNotSupportRequestedMinHistorySize");
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples), eq
                                  "PublishSubscribeOpenError::DoesNotSupportRequestedMinSubscriberBorrowedSamples");
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers), eq
                                  "PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfPublishers");
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers), eq
                                  "PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfSubscribers");
        assert_that!(format!("{}", PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes), eq
                                  "PublishSubscribeOpenError::DoesNotSupportRequestedAmountOfNodes");
        assert_that!(format!("{}", PublishSubscribeOpenError::IncompatibleOverflowBehavior), eq
                                  "PublishSubscribeOpenError::IncompatibleOverflowBehavior");
        assert_that!(format!("{}", PublishSubscribeOpenError::InsufficientPermissions), eq
                                  "PublishSubscribeOpenError::InsufficientPermissions");
        assert_that!(format!("{}", PublishSubscribeOpenError::ServiceInCorruptedState), eq
                                  "PublishSubscribeOpenError::ServiceInCorruptedState");
        assert_that!(format!("{}", PublishSubscribeOpenError::HangsInCreation), eq
                                  "PublishSubscribeOpenError::HangsInCreation");
        assert_that!(format!("{}", PublishSubscribeOpenError::ExceedsMaxNumberOfNodes), eq
                                  "PublishSubscribeOpenError::ExceedsMaxNumberOfNodes");
        assert_that!(format!("{}", PublishSubscribeOpenError::IsMarkedForDestruction), eq
                                  "PublishSubscribeOpenError::IsMarkedForDestruction");
    }

    #[test]
    fn create_error_display_works<Sut: Service>() {
        assert_that!(format!("{}", PublishSubscribeCreateError::ServiceInCorruptedState), eq
                                  "PublishSubscribeCreateError::ServiceInCorruptedState");
        assert_that!(format!("{}", PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize), eq
                                  "PublishSubscribeCreateError::SubscriberBufferMustBeLargerThanHistorySize");
        assert_that!(format!("{}", PublishSubscribeCreateError::AlreadyExists), eq
                                  "PublishSubscribeCreateError::AlreadyExists");
        assert_that!(format!("{}", PublishSubscribeCreateError::InsufficientPermissions), eq
                                  "PublishSubscribeCreateError::InsufficientPermissions");
        assert_that!(format!("{}", PublishSubscribeCreateError::InternalFailure), eq
                                  "PublishSubscribeCreateError::InternalFailure");
        assert_that!(format!("{}", PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance), eq
                                  "PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance");
    }

    #[test]
    fn has_samples_tracks_receivable_samples_in_subscriber<Sut: Service>() {
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .unwrap();

        let subscriber = sut.subscriber_builder().create().unwrap();
        let publisher = sut.publisher_builder().create().unwrap();

        assert_that!(subscriber.has_samples().unwrap(), eq false);
        assert_that!(publisher.send_copy(1234), is_ok);
        assert_that!(subscriber.has_samples().unwrap(), eq true);

        let _ = subscriber.receive().unwrap();

        assert_that!(subscriber.has_samples().unwrap(), eq false);
    }

    #[test]
    fn subscriber_can_still_receive_sample_when_publisher_was_disconnected<Sut: Service>() {
        const NUMBER_OF_SAMPLES: usize = 4;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .subscriber_max_buffer_size(NUMBER_OF_SAMPLES)
            .max_publishers(1)
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        for i in 0..NUMBER_OF_SAMPLES {
            assert_that!(publisher.send_copy(i), is_ok);
        }

        drop(publisher);

        for i in 0..NUMBER_OF_SAMPLES {
            let result = subscriber.receive().unwrap();
            assert_that!(result, is_some);
            let sample = result.unwrap();
            assert_that!(*sample, eq i);
        }
    }

    #[test]
    fn subscriber_disconnected_publisher_does_not_block_new_publishers<Sut: Service>() {
        set_log_level(LogLevel::Error);
        const NUMBER_OF_SAMPLES: usize = 4;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .subscriber_max_buffer_size(NUMBER_OF_SAMPLES)
            .max_publishers(1)
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        for i in 0..NUMBER_OF_SAMPLES {
            assert_that!(publisher.send_copy(i), is_ok);
        }

        drop(publisher);

        let _publisher = sut.publisher_builder().create().unwrap();

        for i in 0..NUMBER_OF_SAMPLES {
            let result = subscriber.receive().unwrap();
            assert_that!(result, is_some);
            let sample = result.unwrap();
            assert_that!(*sample, eq i);
        }
    }

    #[test]
    fn subscriber_acquires_samples_of_disconnected_publisher_first<Sut: Service>() {
        set_log_level(LogLevel::Error);
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<usize>()
            .subscriber_max_buffer_size(2)
            .max_publishers(1)
            .create()
            .unwrap();

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        assert_that!(publisher.send_copy(123), is_ok);

        drop(publisher);

        let publisher = sut.publisher_builder().create().unwrap();
        assert_that!(publisher.send_copy(456), is_ok);

        let sample = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample, eq 123);
        let sample = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample, eq 456);
    }

    #[test]
    fn communication_with_custom_payload_works<Sut: Service>() {
        set_log_level(LogLevel::Error);
        const NUMBER_OF_ELEMENTS: usize = 1;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_details = TypeDetail::new::<u8>(TypeVariant::FixedSize);
        testing::type_detail_set_size(&mut type_details, 1024);
        testing::type_detail_set_alignment(&mut type_details, 1024);

        let sut = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[CustomPayloadMarker]>()
                .user_header::<CustomHeaderMarker>()
                .__internal_set_payload_type_details(&type_details)
                .create()
                .unwrap()
        };

        let publisher = sut.publisher_builder().create().unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        let sample = unsafe { publisher.loan_custom_payload(NUMBER_OF_ELEMENTS).unwrap() };
        assert_that!(sample.payload(), len type_details.size());
        assert_that!((sample.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(sample.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);

        unsafe { sample.assume_init().send().unwrap() };

        let sample = unsafe { subscriber.receive_custom_payload().unwrap().unwrap() };
        assert_that!(sample.payload(), len type_details.size());
        assert_that!((sample.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(sample.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);
    }

    #[test]
    fn communication_with_custom_slice_payload_works<Sut: Service>() {
        set_log_level(LogLevel::Error);
        const NUMBER_OF_ELEMENTS: usize = 7;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let mut type_details = TypeDetail::new::<u8>(TypeVariant::Dynamic);
        testing::type_detail_set_size(&mut type_details, 2048);
        testing::type_detail_set_alignment(&mut type_details, 2048);

        let sut = unsafe {
            node.service_builder(&service_name)
                .publish_subscribe::<[CustomPayloadMarker]>()
                .user_header::<CustomHeaderMarker>()
                .__internal_set_payload_type_details(&type_details)
                .create()
                .unwrap()
        };

        let publisher = sut
            .publisher_builder()
            .initial_max_slice_len(NUMBER_OF_ELEMENTS)
            .create()
            .unwrap();
        let subscriber = sut.subscriber_builder().create().unwrap();

        let sample = unsafe { publisher.loan_custom_payload(NUMBER_OF_ELEMENTS).unwrap() };
        assert_that!(sample.payload(), len type_details.size() * NUMBER_OF_ELEMENTS);
        assert_that!((sample.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(sample.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);

        unsafe { sample.assume_init().send().unwrap() };

        let sample = unsafe { subscriber.receive_custom_payload().unwrap().unwrap() };
        assert_that!(sample.payload(), len type_details.size() * NUMBER_OF_ELEMENTS);
        assert_that!((sample.payload().as_ptr() as usize % type_details.alignment()), eq 0);
        assert_that!(sample.header().number_of_elements(), eq NUMBER_OF_ELEMENTS as u64);
    }

    #[test]
    fn send_increasing_samples_with_static_allocation_strategy_fails<Sut: Service>() {
        const SLICE_SIZE: usize = 1024;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_pub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .create()
            .unwrap();

        let publisher = service_pub
            .publisher_builder()
            .initial_max_slice_len(SLICE_SIZE)
            .allocation_strategy(AllocationStrategy::Static)
            .create()
            .unwrap();

        let sample = publisher.loan_slice(SLICE_SIZE - 1);
        assert_that!(sample, is_ok);

        let sample = publisher.loan_slice(SLICE_SIZE);
        assert_that!(sample, is_ok);

        let sample = publisher.loan_slice(SLICE_SIZE + 1);
        assert_that!(sample, is_err);
        assert_that!(sample.err(), eq Some(LoanError::ExceedsMaxLoanSize));
    }

    fn send_and_receives_increasing_samples_works<Sut: Service>(
        allocation_strategy: AllocationStrategy,
    ) {
        const ITERATIONS: usize = 128;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_pub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .create()
            .unwrap();

        let service_sub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open()
            .unwrap();

        let publisher = service_pub
            .publisher_builder()
            .initial_max_slice_len(1)
            .allocation_strategy(allocation_strategy)
            .create()
            .unwrap();

        let subscriber = service_sub.subscriber_builder().create().unwrap();

        for n in 0..ITERATIONS {
            let sample_size = (n + 1) * 32;
            let mut sample = publisher.loan_slice(sample_size).unwrap();
            for byte in sample.payload_mut() {
                *byte = n as u8;
            }

            sample.send().unwrap();

            let sample = subscriber.receive().unwrap().unwrap();
            assert_that!(sample.payload(), len sample_size);
            for byte in sample.payload() {
                assert_that!(*byte, eq n as u8);
            }
        }
    }

    #[test]
    fn send_and_receives_increasing_samples_works_for_best_fit_allocation_strategy<Sut: Service>() {
        send_and_receives_increasing_samples_works::<Sut>(AllocationStrategy::BestFit);
    }

    #[test]
    fn send_and_receives_increasing_samples_works_for_power_of_two_allocation_strategy<
        Sut: Service,
    >() {
        send_and_receives_increasing_samples_works::<Sut>(AllocationStrategy::PowerOfTwo);
    }

    fn send_and_receives_increasing_samples_with_overflow_works<Sut: Service>(
        allocation_strategy: AllocationStrategy,
    ) {
        const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 5;
        const ITERATIONS: usize = 128;
        const REPETITIONS: usize = 13;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_pub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .subscriber_max_buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
            .create()
            .unwrap();

        let service_sub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open()
            .unwrap();

        let publisher = service_pub
            .publisher_builder()
            .initial_max_slice_len(1)
            .allocation_strategy(allocation_strategy)
            .create()
            .unwrap();

        let subscriber = service_sub.subscriber_builder().create().unwrap();

        for n in 0..ITERATIONS {
            let sample_size = (n + 1) * 32;
            for _ in 0..REPETITIONS {
                let mut sample = publisher.loan_slice(sample_size).unwrap();
                for byte in sample.payload_mut() {
                    *byte = n as u8;
                }

                sample.send().unwrap();
            }

            let sample = subscriber.receive().unwrap().unwrap();
            assert_that!(sample.payload(), len sample_size);
            for byte in sample.payload() {
                assert_that!(*byte, eq n as u8);
            }
        }
    }

    #[test]
    fn send_and_receives_increasing_samples_with_overflow_for_best_fit_allocation_strategy<
        Sut: Service,
    >() {
        send_and_receives_increasing_samples_with_overflow_works::<Sut>(
            AllocationStrategy::BestFit,
        );
    }

    #[test]
    fn send_and_receives_increasing_samples_with_overflow_for_power_of_two_allocation_strategy<
        Sut: Service,
    >() {
        send_and_receives_increasing_samples_with_overflow_works::<Sut>(
            AllocationStrategy::PowerOfTwo,
        );
    }

    fn deliver_history_with_increasing_samples_works<Sut: Service>(
        allocation_strategy: AllocationStrategy,
    ) {
        const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 12;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let service_pub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .subscriber_max_buffer_size(SUBSCRIBER_MAX_BUFFER_SIZE)
            .history_size(SUBSCRIBER_MAX_BUFFER_SIZE)
            .create()
            .unwrap();

        let service_sub = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open()
            .unwrap();

        let publisher = service_pub
            .publisher_builder()
            .initial_max_slice_len(1)
            .allocation_strategy(allocation_strategy)
            .create()
            .unwrap();

        for n in 0..SUBSCRIBER_MAX_BUFFER_SIZE {
            let sample_size = (n + 1) * 32;
            let mut sample = publisher.loan_slice(sample_size).unwrap();
            for byte in sample.payload_mut() {
                *byte = n as u8;
            }

            sample.send().unwrap();
        }

        let subscriber = service_sub.subscriber_builder().create().unwrap();
        publisher.update_connections().unwrap();

        for n in 0..SUBSCRIBER_MAX_BUFFER_SIZE {
            let sample_size = (n + 1) * 32;
            let sample = subscriber.receive().unwrap().unwrap();
            assert_that!(sample.payload(), len sample_size);
            for byte in sample.payload() {
                assert_that!(*byte, eq n as u8);
            }
        }
    }

    #[test]
    fn deliver_history_with_increasing_samples_works_for_best_fit_allocation_strategy<
        Sut: Service,
    >() {
        deliver_history_with_increasing_samples_works::<Sut>(AllocationStrategy::BestFit);
    }

    #[test]
    fn deliver_history_with_increasing_samples_works_for_power_of_two_allocation_strategy<
        Sut: Service,
    >() {
        deliver_history_with_increasing_samples_works::<Sut>(AllocationStrategy::PowerOfTwo);
    }

    #[test]
    fn does_not_leak_when_subscriber_has_smaller_buffer_size_than_history_size<Sut: Service>() {
        let _watchdog = Watchdog::new();
        const HISTORY_SIZE: usize = 1000;
        const REPETITIONS: usize = 10;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let finish_setup = Barrier::new(2);
        let start = Barrier::new(2);
        let end = Barrier::new(2);

        std::thread::scope(|s| {
            let update_connection_thread = s.spawn(|| {
                let service = node
                    .service_builder(&service_name)
                    .publish_subscribe::<usize>()
                    .max_publishers(1)
                    .max_subscribers(1)
                    .subscriber_max_borrowed_samples(1)
                    .history_size(HISTORY_SIZE)
                    .subscriber_max_buffer_size(HISTORY_SIZE)
                    .create()
                    .unwrap();

                let publisher = service
                    .publisher_builder()
                    .max_loaned_samples(1)
                    .create()
                    .unwrap();

                for n in 0..HISTORY_SIZE {
                    publisher.send_copy(n).unwrap();
                }

                finish_setup.wait();

                for _ in 0..REPETITIONS {
                    start.wait();

                    publisher.update_connections().unwrap();

                    end.wait();
                }
            });

            let new_subscriber_thread = s.spawn(|| {
                finish_setup.wait();

                let service = node
                    .service_builder(&service_name)
                    .publish_subscribe::<usize>()
                    .open()
                    .unwrap();

                for _ in 0..REPETITIONS {
                    let subscriber = service
                        .subscriber_builder()
                        .buffer_size(1)
                        .create()
                        .unwrap();
                    start.wait();

                    let mut previous_value = 0;
                    for _ in 0..HISTORY_SIZE {
                        let sample = subscriber.receive().unwrap();
                        if let Some(sample) = sample {
                            assert_that!(*sample, ge previous_value);
                            previous_value = *sample;
                        }
                    }

                    end.wait();
                }
            });

            update_connection_thread.join().unwrap();
            new_subscriber_thread.join().unwrap();
        });
    }

    #[test]
    fn listing_all_publishers_works<S: Service>() {
        const NUMBER_OF_PUBLISHERS: usize = 18;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(NUMBER_OF_PUBLISHERS)
            .create()
            .unwrap();

        let mut publishers = vec![];

        for _ in 0..NUMBER_OF_PUBLISHERS {
            publishers.push(sut.publisher_builder().create().unwrap());
        }

        let mut publisher_details = vec![];
        sut.dynamic_config().list_publishers(|details| {
            publisher_details.push(details.publisher_id);
            CallbackProgression::Continue
        });

        assert_that!(publisher_details, len NUMBER_OF_PUBLISHERS);
        for publisher in publishers {
            assert_that!(publisher_details, contains publisher.id());
        }
    }

    #[test]
    fn listing_all_publishers_stops_on_request<S: Service>() {
        const NUMBER_OF_PUBLISHERS: usize = 16;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_publishers(NUMBER_OF_PUBLISHERS)
            .create()
            .unwrap();

        let mut publishers = vec![];

        for _ in 0..NUMBER_OF_PUBLISHERS {
            publishers.push(sut.publisher_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_publishers(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn listing_all_subscribers_works<S: Service>() {
        const NUMBER_OF_SUBSCRIBERS: usize = 18;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(NUMBER_OF_SUBSCRIBERS)
            .create()
            .unwrap();

        let mut subscribers = vec![];

        for _ in 0..NUMBER_OF_SUBSCRIBERS {
            subscribers.push(sut.subscriber_builder().create().unwrap());
        }

        let mut subscriber_details = vec![];
        sut.dynamic_config().list_subscribers(|details| {
            subscriber_details.push(details.subscriber_id);
            CallbackProgression::Continue
        });

        assert_that!(subscriber_details, len NUMBER_OF_SUBSCRIBERS);
        for subscriber in subscribers {
            assert_that!(subscriber_details, contains subscriber.id());
        }
    }

    #[test]
    fn listing_all_subscribers_stops_on_request<S: Service>() {
        const NUMBER_OF_SUBSCRIBERS: usize = 16;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .max_subscribers(NUMBER_OF_SUBSCRIBERS)
            .create()
            .unwrap();

        let mut subscribers = vec![];

        for _ in 0..NUMBER_OF_SUBSCRIBERS {
            subscribers.push(sut.subscriber_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_subscribers(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn receive_does_not_return_error_when_pub_goes_out_of_scope_after_reallocation<S: Service>() {
        const SLICE_MAX_LEN: usize = 1;
        let service_name = generate_name();
        let config = testing::generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .create()
            .unwrap();

        let subscriber = service.subscriber_builder().create().unwrap();

        let publisher = service
            .publisher_builder()
            .initial_max_slice_len(SLICE_MAX_LEN)
            .allocation_strategy(AllocationStrategy::BestFit)
            .create()
            .unwrap();

        // send and receive once so that the subscriber maps the data segment
        let sample = publisher.loan_slice_uninit(SLICE_MAX_LEN).unwrap();
        sample.write_from_fn(|i| i as u8).send().unwrap();

        let recv_sample = subscriber.receive();
        assert_that!(recv_sample, is_ok);

        // publisher has to reallocate the data segment
        let sample = publisher.loan_slice_uninit(SLICE_MAX_LEN + 4096).unwrap();
        sample.write_from_fn(|i| i as u8).send().unwrap();
        // publisher goes out of scope and closes the reallocated data segment as it was not yet mapped by the
        // subscriber
        drop(publisher);

        let recv_res = subscriber.receive();
        assert_that!(recv_res, is_ok);
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
