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
mod service {
    use std::marker::PhantomData;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Barrier;

    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::event::{EventCreateError, EventOpenError};
    use iceoryx2::service::builder::publish_subscribe::{
        PublishSubscribeCreateError, PublishSubscribeOpenError,
    };
    use iceoryx2::service::port_factory::{event, publish_subscribe};
    use iceoryx2::service::{ServiceDetailsError, ServiceListError};
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    trait SutFactory<Sut: Service>: Send + Sync {
        type Factory: PortFactory;
        type CreateError: std::fmt::Debug;
        type OpenError: std::fmt::Debug;

        fn new() -> Self;
        fn create1(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError>;
        fn open1(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError>;

        fn assert_create_error(error: Self::CreateError);
        fn assert_open_error(error: Self::OpenError);
        fn assert_attribute_error(error: Self::OpenError);
    }

    struct PubSubTests<Sut: Service> {
        _data: PhantomData<Sut>,
    }

    unsafe impl<Sut: Service> Send for PubSubTests<Sut> {}
    unsafe impl<Sut: Service> Sync for PubSubTests<Sut> {}

    struct EventTests<Sut: Service> {
        _data: PhantomData<Sut>,
    }

    unsafe impl<Sut: Service> Send for EventTests<Sut> {}
    unsafe impl<Sut: Service> Sync for EventTests<Sut> {}

    impl<Sut: Service> SutFactory<Sut> for PubSubTests<Sut> {
        type Factory = publish_subscribe::PortFactory<Sut, u64, ()>;
        type CreateError = PublishSubscribeCreateError;
        type OpenError = PublishSubscribeOpenError;

        fn new() -> Self {
            Self { _data: PhantomData }
        }

        fn open1(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError> {
            node.service_builder(service_name.clone())
                .publish_subscribe::<u64>()
                .open_with_attributes(attributes)
        }

        fn create1(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            node.service_builder(service_name.clone())
                .publish_subscribe::<u64>()
                .create_with_attributes(attributes)
        }

        fn assert_attribute_error(error: Self::OpenError) {
            assert_that!(error, eq PublishSubscribeOpenError::IncompatibleAttributes);
        }

        fn assert_create_error(error: Self::CreateError) {
            assert_that!(
                error,
                any_of([
                    PublishSubscribeCreateError::AlreadyExists,
                    PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance,
                ])
            );
        }
        fn assert_open_error(error: Self::OpenError) {
            assert_that!(
                error,
                any_of([
                    PublishSubscribeOpenError::DoesNotExist,
                    PublishSubscribeOpenError::InsufficientPermissions,
                    PublishSubscribeOpenError::ServiceInCorruptedState,
                ])
            );
        }
    }

    impl<Sut: Service> SutFactory<Sut> for EventTests<Sut> {
        type Factory = event::PortFactory<Sut>;
        type CreateError = EventCreateError;
        type OpenError = EventOpenError;

        fn new() -> Self {
            Self { _data: PhantomData }
        }

        fn open1(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError> {
            node.service_builder(service_name.clone())
                .event()
                .open_with_attributes(attributes)
        }

        fn create1(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            node.service_builder(service_name.clone())
                .event()
                .create_with_attributes(attributes)
        }

        fn assert_attribute_error(error: Self::OpenError) {
            assert_that!(error, eq EventOpenError::IncompatibleAttributes);
        }

        fn assert_create_error(error: Self::CreateError) {
            assert_that!(
                error,
                any_of([
                    EventCreateError::AlreadyExists,
                    EventCreateError::IsBeingCreatedByAnotherInstance,
                ])
            );
        }
        fn assert_open_error(error: Self::OpenError) {
            assert_that!(
                error,
                any_of([
                    EventOpenError::DoesNotExist,
                    EventOpenError::InsufficientPermissions,
                    EventOpenError::ServiceInCorruptedState,
                ])
            );
        }
    }

    #[test]
    fn same_name_with_different_messaging_pattern_is_allowed<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let sut_pub_sub = node_1
            .service_builder(service_name.clone())
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut_pub_sub, is_ok);
        let sut_pub_sub = sut_pub_sub.unwrap();

        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let sut_event = node_2.service_builder(service_name).event().create();
        assert_that!(sut_event, is_ok);
        let sut_event = sut_event.unwrap();

        let sut_subscriber = sut_pub_sub.subscriber_builder().create().unwrap();
        let sut_publisher = sut_pub_sub.publisher_builder().create().unwrap();

        let sut_listener = sut_event.listener_builder().create().unwrap();
        let sut_notifier = sut_event.notifier_builder().create().unwrap();

        const SAMPLE_VALUE: u64 = 891231211;
        sut_publisher.send_copy(SAMPLE_VALUE).unwrap();
        let received_sample = sut_subscriber.receive().unwrap().unwrap();
        assert_that!(*received_sample, eq(SAMPLE_VALUE));

        const EVENT_ID: EventId = EventId::new(31);
        sut_notifier.notify_with_custom_event_id(EVENT_ID).unwrap();
        let received_event = sut_listener.try_wait_one().unwrap();
        assert_that!(received_event, eq Some(EVENT_ID));
    }

    #[test]
    fn concurrent_creating_services_with_unique_names_is_successful<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let _watch_dog = Watchdog::new();
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024) * 2;
        const NUMBER_OF_ITERATIONS: usize = 50;
        let test = Factory::new();

        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_threads {
                threads.push(s.spawn(|| {
                    let node = NodeBuilder::new().create::<Sut>().unwrap();
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let service_name = generate_name();
                        barrier_enter.wait();

                        let _sut = test
                            .create1(&node, &service_name, &AttributeSpecifier::new())
                            .unwrap();

                        barrier_exit.wait();
                    }
                }));
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });
    }

    #[test]
    fn concurrent_creating_services_with_same_name_fails_for_all_but_one<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let _watch_dog = Watchdog::new();
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024) * 2;
        const NUMBER_OF_ITERATIONS: usize = 50;
        let test = Factory::new();

        let success_counter = AtomicU64::new(0);
        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);
        let service_name = generate_name();

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_threads {
                threads.push(s.spawn(|| {
                    let node = NodeBuilder::new().create::<Sut>().unwrap();
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        barrier_enter.wait();

                        let sut = test.create1(&node, &service_name, &AttributeSpecifier::new());
                        match sut {
                            Ok(_) => {
                                success_counter.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => {
                                Factory::assert_create_error(e);
                            }
                        }

                        barrier_exit.wait();
                    }
                }));
            }

            for thread in threads {
                thread.join().unwrap();
            }

            assert_that!(
                success_counter.load(Ordering::Relaxed),
                eq(NUMBER_OF_ITERATIONS as u64)
            );
        });
    }

    #[test]
    fn concurrent_opening_and_closing_services_with_same_name_is_handled_gracefully<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let _watch_dog = Watchdog::new();
        const NUMBER_OF_CLOSE_THREADS: usize = 1;
        let number_of_open_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024) * 2;
        let number_of_threads = NUMBER_OF_CLOSE_THREADS + number_of_open_threads;
        let test = Factory::new();

        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);

        const NUMBER_OF_ITERATIONS: usize = 50;
        let service_names: Vec<_> = (0..NUMBER_OF_ITERATIONS).map(|_| generate_name()).collect();
        let service_names = &service_names;

        std::thread::scope(|s| {
            let mut threads = vec![];
            threads.push(s.spawn(|| {
                let node = NodeBuilder::new().create::<Sut>().unwrap();
                for service_name in service_names {
                    let sut = test
                        .create1(&node, &service_name, &AttributeSpecifier::new())
                        .unwrap();
                    barrier_enter.wait();

                    drop(sut);

                    barrier_exit.wait();
                }
            }));

            for _ in 0..number_of_open_threads {
                threads.push(s.spawn(|| {
                    let node = NodeBuilder::new().create::<Sut>().unwrap();
                    for service_name in service_names {
                        barrier_enter.wait();

                        let sut = test.open1(&node, &service_name, &AttributeVerifier::new());
                        match sut {
                            Ok(_) => (),
                            Err(e) => {
                                Factory::assert_open_error(e);
                            }
                        }

                        barrier_exit.wait();
                    }
                }));
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });
    }

    #[test]
    fn setting_attributes_in_creator_can_be_read_in_opener<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_name();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("2. No more", "Coffee")
            .define("3. Just have a", "lick on the toad");
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        assert_that!(sut_create.attributes(), eq defined_attributes.attributes());

        let sut_open = test
            .open1(&node_2, &service_name, &AttributeVerifier::new())
            .unwrap();

        assert_that!(sut_open.attributes(), eq defined_attributes.attributes());
    }

    #[test]
    fn opener_succeeds_when_attributes_do_match<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("1. Hello", "Take a number")
            .define("2. No more", "Coffee")
            .define("3. Just have a", "lick on the toad");

        let _sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open1(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require("1. Hello", "Hypnotoad")
                .require("1. Hello", "Take a number")
                .require("3. Just have a", "lick on the toad"),
        );

        assert_that!(sut_open, is_ok);
        let sut_open = sut_open.unwrap();

        assert_that!(sut_open.attributes(), eq defined_attributes.attributes());
    }

    #[test]
    fn opener_fails_when_attribute_value_does_not_match<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("2. No more", "Coffee");
        let _sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open1(
            &node_2,
            &service_name,
            &AttributeVerifier::new().require("1. Hello", "lick on the toad"),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[test]
    fn opener_fails_when_attribute_key_does_not_exist<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("2. No more", "Coffee");
        let _sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open1(
            &node_2,
            &service_name,
            &AttributeVerifier::new().require("Whatever", "lick on the toad"),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[test]
    fn opener_fails_when_attribute_value_does_not_exist<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("1. Hello", "Number Two")
            .define("2. No more", "Coffee");
        let _sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open1(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require("1. Hello", "lick on the toad")
                .require("1. Hello", "Number Eight"),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[test]
    fn opener_fails_when_attribute_required_key_does_not_exist<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("2. No more", "Coffee");
        let _sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open1(
            &node_2,
            &service_name,
            &AttributeVerifier::new().require_key("i do not exist"),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[test]
    fn opener_succeeds_when_attribute_required_key_does_exist<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_name();
        let node_1 = NodeBuilder::new().create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let defined_attributes = AttributeSpecifier::new()
            .define("1. Hello", "Hypnotoad")
            .define("2. No more", "Coffee");
        let _sut_create = test
            .create1(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open1(
            &node_2,
            &service_name,
            &AttributeVerifier::new().require_key("2. No more"),
        );

        assert_that!(sut_open, is_ok);
    }

    #[test]
    fn details_error_display_works<Sut: Service, Factory: SutFactory<Sut>>() {
        assert_that!(format!("{}", ServiceDetailsError::FailedToOpenStaticServiceInfo), eq
                                  "ServiceDetailsError::FailedToOpenStaticServiceInfo");

        assert_that!(format!("{}", ServiceDetailsError::FailedToReadStaticServiceInfo), eq
                                  "ServiceDetailsError::FailedToReadStaticServiceInfo");

        assert_that!(format!("{}", ServiceDetailsError::FailedToDeserializeStaticServiceInfo), eq
                                  "ServiceDetailsError::FailedToDeserializeStaticServiceInfo");

        assert_that!(format!("{}", ServiceDetailsError::ServiceInInconsistentState), eq
                                  "ServiceDetailsError::ServiceInInconsistentState");

        assert_that!(format!("{}", ServiceDetailsError::VersionMismatch), eq
                                  "ServiceDetailsError::VersionMismatch");

        assert_that!(format!("{}", ServiceDetailsError::InternalError), eq
                                  "ServiceDetailsError::InternalError");

        assert_that!(format!("{}", ServiceDetailsError::FailedToAcquireNodeState), eq
                                  "ServiceDetailsError::FailedToAcquireNodeState");
    }

    #[test]
    fn list_error_display_works<Sut: Service, Factory: SutFactory<Sut>>() {
        assert_that!(format!("{}", ServiceListError::InsufficientPermissions), eq
                                  "ServiceListError::InsufficientPermissions");

        assert_that!(format!("{}", ServiceListError::InternalError), eq
                                  "ServiceListError::InternalError");
    }

    mod zero_copy {
        use iceoryx2::service::zero_copy::Service;

        #[instantiate_tests(<Service, crate::service::EventTests::<Service>>)]
        mod event {}

        #[instantiate_tests(<Service, crate::service::PubSubTests::<Service>>)]
        mod publish_subscribe {}
    }

    mod process_local {
        use iceoryx2::service::process_local::Service;

        #[instantiate_tests(<Service, crate::service::EventTests::<Service>>)]
        mod event {}

        #[instantiate_tests(<Service, crate::service::PubSubTests::<Service>>)]
        mod publish_subscribe {}
    }
}
