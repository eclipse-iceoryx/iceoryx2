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
    use std::time::Duration;

    use iceoryx2::node::NodeView;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::event::{EventCreateError, EventOpenError};
    use iceoryx2::service::builder::publish_subscribe::{
        PublishSubscribeCreateError, PublishSubscribeOpenError,
    };
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::service::port_factory::{event, publish_subscribe};
    use iceoryx2::service::{ServiceDetailsError, ServiceListError};
    use iceoryx2_bb_log::{set_log_level, LogLevel};
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
        fn create(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError>;
        fn open(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError>;
        fn messaging_pattern() -> MessagingPattern;

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

        fn open(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError> {
            node.service_builder(service_name)
                .publish_subscribe::<u64>()
                .open_with_attributes(attributes)
        }

        fn create(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            let number_of_nodes = (SystemInfo::NumberOfCpuCores.value()).clamp(128, 1024);
            node.service_builder(service_name)
                .publish_subscribe::<u64>()
                .max_nodes(number_of_nodes)
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
                    PublishSubscribeCreateError::HangsInCreation
                ])
            );
        }
        fn assert_open_error(error: Self::OpenError) {
            assert_that!(
                error,
                any_of([
                    PublishSubscribeOpenError::DoesNotExist,
                    PublishSubscribeOpenError::InsufficientPermissions,
                    PublishSubscribeOpenError::IsMarkedForDestruction,
                    PublishSubscribeOpenError::ServiceInCorruptedState,
                    PublishSubscribeOpenError::HangsInCreation
                ])
            );
        }

        fn messaging_pattern() -> MessagingPattern {
            MessagingPattern::PublishSubscribe
        }
    }

    impl<Sut: Service> SutFactory<Sut> for EventTests<Sut> {
        type Factory = event::PortFactory<Sut>;
        type CreateError = EventCreateError;
        type OpenError = EventOpenError;

        fn new() -> Self {
            Self { _data: PhantomData }
        }

        fn open(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError> {
            node.service_builder(service_name)
                .event()
                .open_with_attributes(attributes)
        }

        fn create(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            let number_of_nodes = (SystemInfo::NumberOfCpuCores.value()).clamp(128, 1024);
            node.service_builder(service_name)
                .event()
                .max_nodes(number_of_nodes)
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
                    EventCreateError::HangsInCreation,
                ])
            );
        }
        fn assert_open_error(error: Self::OpenError) {
            assert_that!(
                error,
                any_of([
                    EventOpenError::DoesNotExist,
                    EventOpenError::InsufficientPermissions,
                    EventOpenError::IsMarkedForDestruction,
                    EventOpenError::ServiceInCorruptedState,
                    EventOpenError::HangsInCreation
                ])
            );
        }

        fn messaging_pattern() -> MessagingPattern {
            MessagingPattern::Event
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
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut_pub_sub, is_ok);
        let sut_pub_sub = sut_pub_sub.unwrap();

        let node_2 = NodeBuilder::new().create::<Sut>().unwrap();
        let sut_event = node_2.service_builder(&service_name).event().create();
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
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        const NUMBER_OF_ITERATIONS: usize = 25;
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
                            .create(&node, &service_name, &AttributeSpecifier::new())
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
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        const NUMBER_OF_ITERATIONS: usize = 25;
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

                        let sut = test.create(&node, &service_name, &AttributeSpecifier::new());
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
        set_log_level(LogLevel::Debug);
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));
        const NUMBER_OF_CLOSE_THREADS: usize = 1;
        let number_of_open_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        let number_of_threads = NUMBER_OF_CLOSE_THREADS + number_of_open_threads;
        let test = Factory::new();

        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);

        const NUMBER_OF_ITERATIONS: usize = 100;
        let service_names: Vec<_> = (0..NUMBER_OF_ITERATIONS).map(|_| generate_name()).collect();
        let service_names = &service_names;

        std::thread::scope(|s| {
            let mut threads = vec![];
            threads.push(s.spawn(|| {
                let node = NodeBuilder::new().create::<Sut>().unwrap();
                for service_name in service_names {
                    let sut = test
                        .create(&node, &service_name, &AttributeSpecifier::new())
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
                        let sut = test.open(&node, &service_name, &AttributeVerifier::new());

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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        assert_that!(sut_create.attributes(), eq defined_attributes.attributes());

        let sut_open = test
            .open(&node_2, &service_name, &AttributeVerifier::new())
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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
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
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
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

    #[test]
    fn list_services_works<Sut: Service, Factory: SutFactory<Sut>>() {
        const NUMBER_OF_SERVICES: usize = 16;
        let test = Factory::new();

        let mut services = vec![];
        let mut service_ids = vec![];
        let mut nodes = vec![];
        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();
            let node = NodeBuilder::new().create::<Sut>().unwrap();
            let sut = test
                .create(&node, &service_name, &AttributeSpecifier::new())
                .unwrap();

            service_ids.push(sut.uuid().to_string());
            services.push(sut);
            nodes.push(node);
        }

        let mut listed_services = vec![];
        let result = Sut::list(Config::global_config(), |service| {
            listed_services.push(service.static_details.uuid().to_string());
            CallbackProgression::Continue
        });
        assert_that!(result, is_ok);

        for s in listed_services {
            assert_that!(service_ids, contains s);
        }
    }

    #[test]
    fn list_services_stops_when_callback_progression_states_stop<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        const NUMBER_OF_SERVICES: usize = 16;
        let test = Factory::new();
        let node = NodeBuilder::new().create::<Sut>().unwrap();

        let mut services = vec![];
        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();
            let sut = test
                .create(&node, &service_name, &AttributeSpecifier::new())
                .unwrap();

            services.push(sut);
        }

        let mut service_counter = 0;
        let result = Sut::list(Config::global_config(), |_service| {
            service_counter += 1;
            CallbackProgression::Stop
        });
        assert_that!(result, is_ok);
        assert_that!(service_counter, eq 1);
    }

    #[test]
    fn concurrent_service_creation_and_listing_works<Sut: Service, Factory: SutFactory<Sut>>() {
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));
        let test = Factory::new();
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        const NUMBER_OF_ITERATIONS: usize = 40;
        let barrier = Barrier::new(number_of_creators);

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_creators {
                threads.push(s.spawn(|| {
                    let node = NodeBuilder::new().create::<Sut>().unwrap();
                    barrier.wait();

                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let service_name = generate_name();
                        let sut = test
                            .create(&node, &service_name, &AttributeSpecifier::new())
                            .unwrap();

                        let mut found_me = false;
                        let result = Sut::list(Config::global_config(), |s| {
                            if sut.uuid() == s.static_details.uuid() {
                                found_me = true;
                            }
                            CallbackProgression::Continue
                        });

                        assert_that!(result, is_ok);
                        assert_that!(found_me, eq true);
                    }
                }));
            }

            for t in threads {
                t.join().unwrap();
            }
        });
    }

    #[test]
    fn concurrent_node_attaching_to_service_and_listing_works<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));
        let test = Factory::new();
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        const NUMBER_OF_ITERATIONS: usize = 30;
        let barrier = Barrier::new(number_of_creators);

        let main_node = NodeBuilder::new().create::<Sut>().unwrap();
        let service_name = generate_name();
        let attributes = AttributeVerifier::new();
        let _service = test.create(&main_node, &service_name, &AttributeSpecifier::new());

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_creators {
                threads.push(s.spawn(|| {
                    barrier.wait();

                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let node = NodeBuilder::new().create::<Sut>().unwrap();
                        let service = test.open(&node, &service_name, &attributes).unwrap();

                        let mut found_me = false;
                        let result = service.nodes(|node_state| {
                            match node_state {
                                NodeState::Alive(view) => {
                                    if view.id() == node.id() {
                                        found_me = true;
                                    }
                                }
                                NodeState::Dead(view) => {
                                    if view.id() == node.id() {
                                        found_me = true;
                                    }
                                }
                                NodeState::Inaccessible(node_id) => {
                                    if node_id == *node.id() {
                                        found_me = true;
                                    }
                                }
                                NodeState::Undefined(_) => {
                                    assert_that!(true, eq false);
                                }
                            }
                            CallbackProgression::Continue
                        });

                        assert_that!(result, is_ok);
                        assert_that!(found_me, eq true);
                    }
                }));
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });
    }

    #[test]
    fn concurrent_node_attaching_to_service_and_details_node_listing_works<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));
        let test = Factory::new();
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        const NUMBER_OF_ITERATIONS: usize = 30;
        let barrier = Barrier::new(number_of_creators);

        let main_node = NodeBuilder::new().create::<Sut>().unwrap();
        let service_name = generate_name();
        let attributes = AttributeVerifier::new();
        let _service = test.create(&main_node, &service_name, &AttributeSpecifier::new());

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_creators {
                threads.push(s.spawn(|| {
                    barrier.wait();

                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let node = NodeBuilder::new().create::<Sut>().unwrap();
                        let _service = test.open(&node, &service_name, &attributes).unwrap();

                        let service_details = Sut::details(
                            &service_name,
                            Config::global_config(),
                            Factory::messaging_pattern(),
                        )
                        .unwrap()
                        .unwrap();

                        assert_that!(service_details.dynamic_details, is_some);
                        let dynamic_details = service_details.dynamic_details.unwrap();

                        let mut found_me = false;
                        for node_state in dynamic_details.nodes {
                            match node_state {
                                NodeState::Alive(view) => {
                                    if view.id() == node.id() {
                                        found_me = true;
                                    }
                                }
                                NodeState::Dead(view) => {
                                    if view.id() == node.id() {
                                        found_me = true;
                                    }
                                }
                                NodeState::Inaccessible(node_id) => {
                                    if node_id == *node.id() {
                                        found_me = true;
                                    }
                                }
                                NodeState::Undefined(_) => {
                                    assert_that!(true, eq false);
                                }
                            }
                        }
                        assert_that!(found_me, eq true);
                    }
                }));
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });
    }

    #[test]
    fn node_listing_works<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        const NUMBER_OF_NODES: usize = 5;

        let main_node = NodeBuilder::new().create::<Sut>().unwrap();
        let service_name = generate_name();
        let attributes = AttributeVerifier::new();
        let main_service = test
            .create(&main_node, &service_name, &AttributeSpecifier::new())
            .unwrap();

        let mut nodes = vec![];
        let mut node_ids = vec![];
        let mut services = vec![];
        node_ids.push(main_node.id().clone());

        let get_registered_node_ids = |service: &Factory::Factory| {
            let mut registered_node_ids = vec![];
            service
                .nodes(|node_state| {
                    match node_state {
                        NodeState::Alive(view) => registered_node_ids.push(view.id().clone()),
                        NodeState::Dead(view) => registered_node_ids.push(view.id().clone()),
                        NodeState::Inaccessible(_) | NodeState::Undefined(_) => {
                            assert_that!(true, eq false)
                        }
                    }
                    CallbackProgression::Continue
                })
                .unwrap();
            registered_node_ids
        };

        for _ in 0..NUMBER_OF_NODES {
            let node = NodeBuilder::new().create::<Sut>().unwrap();
            let service = test.open(&node, &service_name, &attributes).unwrap();

            let registered_node_ids = get_registered_node_ids(&service);

            node_ids.push(node.id().clone());
            nodes.push(node);
            services.push(service);

            assert_that!(registered_node_ids, len node_ids.len());
            for id in registered_node_ids {
                assert_that!(node_ids, contains id);
            }
        }

        for _ in 0..NUMBER_OF_NODES {
            services.pop();
            nodes.pop();
            node_ids.pop();

            let registered_node_ids = get_registered_node_ids(&main_service);
            assert_that!(registered_node_ids, len node_ids.len());
            for id in registered_node_ids {
                assert_that!(node_ids, contains id);
            }
        }
    }

    #[test]
    fn node_can_open_same_service_without_limits<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_name();
        const REPETITIONS: usize = 128;

        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let sut = test.create(&node, &service_name, &AttributeSpecifier::new());
        assert_that!(sut, is_ok);

        let mut services = vec![];
        services.push(sut.unwrap());

        for _ in 0..REPETITIONS {
            let sut = test.open(&node, &service_name, &AttributeVerifier::new());
            assert_that!(sut, is_ok);
            services.push(sut.unwrap());
        }
    }

    #[test]
    fn uuid_is_equal_in_within_all_opened_instances<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_name();
        let node = NodeBuilder::new().create::<Sut>().unwrap();

        let sut = test
            .create(&node, &service_name, &AttributeSpecifier::new())
            .unwrap();
        let sut2 = test
            .open(&node, &service_name, &AttributeVerifier::new())
            .unwrap();

        assert_that!(sut.uuid(), eq sut2.uuid());
    }

    mod ipc {
        use iceoryx2::service::ipc::Service;

        #[instantiate_tests(<Service, crate::service::EventTests::<Service>>)]
        mod event {}

        #[instantiate_tests(<Service, crate::service::PubSubTests::<Service>>)]
        mod publish_subscribe {}
    }

    mod local {
        use iceoryx2::service::local::Service;

        #[instantiate_tests(<Service, crate::service::EventTests::<Service>>)]
        mod event {}

        #[instantiate_tests(<Service, crate::service::PubSubTests::<Service>>)]
        mod publish_subscribe {}
    }
}
