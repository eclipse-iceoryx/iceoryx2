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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod service {
    use alloc::{format, vec, vec::Vec};
    use core::marker::PhantomData;
    use core::time::Duration;
    use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;

    use iceoryx2::node::NodeView;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::blackboard::{BlackboardCreateError, BlackboardOpenError};
    use iceoryx2::service::builder::event::{EventCreateError, EventOpenError};
    use iceoryx2::service::builder::publish_subscribe::{
        PublishSubscribeCreateError, PublishSubscribeOpenError,
    };
    use iceoryx2::service::builder::request_response::{
        RequestResponseCreateError, RequestResponseOpenError,
    };
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::service::port_factory::{blackboard, event, publish_subscribe, request_response};
    use iceoryx2::service::{ServiceDetailsError, ServiceListError};
    use iceoryx2_bb_concurrency::atomic::AtomicU64;
    use iceoryx2_bb_concurrency::atomic::Ordering;
    use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle};
    use iceoryx2_bb_posix::ipc_capable::Handle;
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_testing::*;

    pub trait SutFactory<Sut: Service>: Send + Sync {
        type Factory: PortFactory + Abandonable;
        type CreateError: core::fmt::Debug;
        type OpenError: core::fmt::Debug;

        fn new() -> Self;
        fn new_with_custom_watchdog(watchdog: Watchdog) -> Self;
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
        fn set_max_number_of_nodes(&mut self, value: usize);
        fn context(&self) -> &Test<Sut>;
        fn context_mut(&mut self) -> &mut Test<Sut>;

        fn assert_create_error(error: Self::CreateError);
        fn assert_open_error(error: Self::OpenError);
        fn assert_attribute_error(error: Self::OpenError);
    }

    pub struct PubSubTests<Sut: Service> {
        pub context: Test<Sut>,
        pub number_of_nodes: usize,
        _data: PhantomData<Sut>,
    }

    unsafe impl<Sut: Service> Send for PubSubTests<Sut> {}
    unsafe impl<Sut: Service> Sync for PubSubTests<Sut> {}

    pub struct EventTests<Sut: Service> {
        pub context: Test<Sut>,
        pub number_of_nodes: usize,
        _data: PhantomData<Sut>,
    }

    unsafe impl<Sut: Service> Send for EventTests<Sut> {}
    unsafe impl<Sut: Service> Sync for EventTests<Sut> {}

    pub struct RequestResponseTests<Sut: Service> {
        pub context: Test<Sut>,
        pub number_of_nodes: usize,
        _data: PhantomData<Sut>,
    }

    unsafe impl<Sut: Service> Send for RequestResponseTests<Sut> {}
    unsafe impl<Sut: Service> Sync for RequestResponseTests<Sut> {}

    pub struct BlackboardTests<Sut: Service> {
        pub context: Test<Sut>,
        pub number_of_nodes: usize,
        _data: PhantomData<Sut>,
    }

    unsafe impl<Sut: Service> Send for BlackboardTests<Sut> {}
    unsafe impl<Sut: Service> Sync for BlackboardTests<Sut> {}

    impl<Sut: Service> SutFactory<Sut> for PubSubTests<Sut> {
        type Factory = publish_subscribe::PortFactory<Sut, u64, ()>;
        type CreateError = PublishSubscribeCreateError;
        type OpenError = PublishSubscribeOpenError;

        fn new() -> Self {
            Self::new_with_custom_watchdog(Watchdog::new())
        }

        fn new_with_custom_watchdog(watchdog: Watchdog) -> Self {
            Self {
                context: Test::new_with_custom_watchdog(watchdog),
                number_of_nodes: (SystemInfo::NumberOfCpuCores.value()).clamp(128, 1024),
                _data: PhantomData,
            }
        }

        fn context(&self) -> &Test<Sut> {
            &self.context
        }

        fn context_mut(&mut self) -> &mut Test<Sut> {
            &mut self.context
        }

        fn set_max_number_of_nodes(&mut self, value: usize) {
            self.number_of_nodes = value;
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
            node.service_builder(service_name)
                .publish_subscribe::<u64>()
                .max_nodes(self.number_of_nodes)
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
                    PublishSubscribeCreateError::HangsInCreation,
                    PublishSubscribeCreateError::ServiceInCorruptedState,
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
            Self::new_with_custom_watchdog(Watchdog::new())
        }

        fn new_with_custom_watchdog(watchdog: Watchdog) -> Self {
            Self {
                context: Test::new_with_custom_watchdog(watchdog),
                number_of_nodes: (SystemInfo::NumberOfCpuCores.value()).clamp(128, 1024),
                _data: PhantomData,
            }
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

        fn context(&self) -> &Test<Sut> {
            &self.context
        }

        fn context_mut(&mut self) -> &mut Test<Sut> {
            &mut self.context
        }

        fn set_max_number_of_nodes(&mut self, value: usize) {
            self.number_of_nodes = value;
        }

        fn create(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            node.service_builder(service_name)
                .event()
                .max_nodes(self.number_of_nodes)
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
                    EventCreateError::ServiceInCorruptedState,
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

    impl<Sut: Service> SutFactory<Sut> for RequestResponseTests<Sut> {
        type Factory = request_response::PortFactory<Sut, u64, (), u64, ()>;
        type CreateError = RequestResponseCreateError;
        type OpenError = RequestResponseOpenError;

        fn new() -> Self {
            Self::new_with_custom_watchdog(Watchdog::new())
        }

        fn new_with_custom_watchdog(watchdog: Watchdog) -> Self {
            Self {
                context: Test::new_with_custom_watchdog(watchdog),
                number_of_nodes: (SystemInfo::NumberOfCpuCores.value()).clamp(128, 1024),
                _data: PhantomData,
            }
        }

        fn open(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError> {
            node.service_builder(service_name)
                .request_response::<u64, u64>()
                .open_with_attributes(attributes)
        }

        fn create(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            node.service_builder(service_name)
                .request_response::<u64, u64>()
                .max_nodes(self.number_of_nodes)
                .create_with_attributes(attributes)
        }

        fn context(&self) -> &Test<Sut> {
            &self.context
        }

        fn context_mut(&mut self) -> &mut Test<Sut> {
            &mut self.context
        }

        fn set_max_number_of_nodes(&mut self, value: usize) {
            self.number_of_nodes = value;
        }

        fn assert_attribute_error(error: Self::OpenError) {
            assert_that!(error, eq RequestResponseOpenError::IncompatibleAttributes);
        }

        fn assert_create_error(error: Self::CreateError) {
            assert_that!(
                error,
                any_of([
                    RequestResponseCreateError::AlreadyExists,
                    RequestResponseCreateError::IsBeingCreatedByAnotherInstance,
                    RequestResponseCreateError::HangsInCreation,
                    RequestResponseCreateError::ServiceInCorruptedState
                ])
            );
        }
        fn assert_open_error(error: Self::OpenError) {
            assert_that!(
                error,
                any_of([
                    RequestResponseOpenError::DoesNotExist,
                    RequestResponseOpenError::InsufficientPermissions,
                    RequestResponseOpenError::IsMarkedForDestruction,
                    RequestResponseOpenError::ServiceInCorruptedState,
                    RequestResponseOpenError::HangsInCreation
                ])
            );
        }

        fn messaging_pattern() -> MessagingPattern {
            MessagingPattern::RequestResponse
        }
    }

    impl<Sut: Service> SutFactory<Sut> for BlackboardTests<Sut> {
        type Factory = blackboard::PortFactory<Sut, u64>;
        type CreateError = BlackboardCreateError;
        type OpenError = BlackboardOpenError;

        fn new() -> Self {
            Self::new_with_custom_watchdog(Watchdog::new())
        }

        fn new_with_custom_watchdog(watchdog: Watchdog) -> Self {
            Self {
                context: Test::new_with_custom_watchdog(watchdog),
                number_of_nodes: (SystemInfo::NumberOfCpuCores.value()).clamp(128, 1024),
                _data: PhantomData,
            }
        }

        fn open(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeVerifier,
        ) -> Result<Self::Factory, Self::OpenError> {
            node.service_builder(service_name)
                .blackboard_opener::<u64>()
                .open_with_attributes(attributes)
        }

        fn create(
            &self,
            node: &Node<Sut>,
            service_name: &ServiceName,
            attributes: &AttributeSpecifier,
        ) -> Result<Self::Factory, Self::CreateError> {
            node.service_builder(service_name)
                .blackboard_creator::<u64>()
                .max_nodes(self.number_of_nodes)
                .add::<u32>(0, 0)
                .create_with_attributes(attributes)
        }

        fn context(&self) -> &Test<Sut> {
            &self.context
        }

        fn context_mut(&mut self) -> &mut Test<Sut> {
            &mut self.context
        }

        fn set_max_number_of_nodes(&mut self, value: usize) {
            self.number_of_nodes = value;
        }

        fn assert_attribute_error(error: Self::OpenError) {
            assert_that!(error, eq BlackboardOpenError::IncompatibleAttributes);
        }

        fn assert_create_error(error: Self::CreateError) {
            assert_that!(
                error,
                any_of([
                    BlackboardCreateError::AlreadyExists,
                    BlackboardCreateError::HangsInCreation,
                    BlackboardCreateError::IsBeingCreatedByAnotherInstance,
                    BlackboardCreateError::ServiceInCorruptedState
                ])
            );
        }
        fn assert_open_error(error: Self::OpenError) {
            assert_that!(
                error,
                any_of([
                    BlackboardOpenError::DoesNotExist,
                    BlackboardOpenError::HangsInCreation,
                    BlackboardOpenError::IsMarkedForDestruction,
                    BlackboardOpenError::ServiceInCorruptedState
                ])
            )
        }

        fn messaging_pattern() -> MessagingPattern {
            MessagingPattern::Blackboard
        }
    }

    #[conformance_test]
    pub fn same_name_with_different_messaging_pattern_is_allowed<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let sut_pub_sub = node_1
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(sut_pub_sub, is_ok);
        let sut_pub_sub = sut_pub_sub.unwrap();

        let node_2 = test.context().create_node();
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

    #[conformance_test]
    pub fn concurrent_creating_services_with_unique_names_is_successful<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test =
            Factory::new_with_custom_watchdog(Watchdog::new_with_timeout(Duration::from_secs(60)));
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        const NUMBER_OF_ITERATIONS: usize = 25;

        let handle_start = BarrierHandle::new();
        let handle_enter = BarrierHandle::new();
        let handle_exit = BarrierHandle::new();
        let barrier_start = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_start)
            .unwrap();
        let barrier_enter = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_enter)
            .unwrap();
        let barrier_exit = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_exit)
            .unwrap();

        thread_scope(|s| {
            for _ in 0..number_of_threads {
                s.thread_builder().spawn(|| {
                    barrier_start.wait();
                    let node = test.context().create_node();
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let service_name = generate_service_name();
                        barrier_enter.wait();

                        let _sut = test
                            .create(&node, &service_name, &AttributeSpecifier::new())
                            .unwrap();

                        barrier_exit.wait();
                    }
                })?;
            }

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn concurrent_creating_services_with_same_name_fails_for_all_but_one<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        const NUMBER_OF_ITERATIONS: usize = 25;

        let success_counter = AtomicU64::new(0);
        let handle_enter = BarrierHandle::new();
        let handle_exit = BarrierHandle::new();
        let barrier_enter = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_enter)
            .unwrap();
        let barrier_exit = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_exit)
            .unwrap();
        let service_name = generate_service_name();

        thread_scope(|s| {
            for _ in 0..number_of_threads {
                s.thread_builder().spawn(|| {
                    let node = test.context().create_node();
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
                })?;
            }

            Ok(())
        })
        .unwrap();

        assert_that!(
            success_counter.load(Ordering::Relaxed),
            eq(NUMBER_OF_ITERATIONS as u64)
        );
    }

    #[conformance_test]
    pub fn concurrent_opening_and_closing_services_with_same_name_is_handled_gracefully<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test =
            Factory::new_with_custom_watchdog(Watchdog::new_with_timeout(Duration::from_secs(120)));
        const NUMBER_OF_CLOSE_THREADS: usize = 1;
        let number_of_open_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        let number_of_threads = NUMBER_OF_CLOSE_THREADS + number_of_open_threads;

        let handle_enter = BarrierHandle::new();
        let handle_exit = BarrierHandle::new();
        let barrier_enter = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_enter)
            .unwrap();
        let barrier_exit = BarrierBuilder::new(number_of_threads as _)
            .create(&handle_exit)
            .unwrap();

        const NUMBER_OF_ITERATIONS: usize = 100;
        let service_names: Vec<_> = (0..NUMBER_OF_ITERATIONS)
            .map(|_| generate_service_name())
            .collect();
        let service_names = &service_names;

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let node = test.context().create_node();
                for service_name in service_names {
                    let sut = test
                        .create(&node, service_name, &AttributeSpecifier::new())
                        .unwrap();

                    barrier_enter.wait();
                    barrier_exit.wait();
                    drop(sut);
                }
            })?;

            for _ in 0..number_of_open_threads {
                s.thread_builder().spawn(|| {
                    let node = test.context().create_node();
                    for service_name in service_names {
                        barrier_enter.wait();
                        let sut = test.open(&node, service_name, &AttributeVerifier::new());

                        match sut {
                            Ok(_) => (),
                            Err(e) => {
                                Factory::assert_open_error(e);
                            }
                        }

                        barrier_exit.wait();
                    }
                })?;
            }

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn setting_attributes_in_creator_can_be_read_in_opener<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"3. Just have a".try_into().unwrap(),
                &"lick on the toad".try_into().unwrap(),
            )
            .unwrap();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        assert_that!(sut_create.attributes(), eq defined_attributes.attributes());

        let sut_open = test
            .open(&node_2, &service_name, &AttributeVerifier::new())
            .unwrap();

        assert_that!(sut_open.attributes(), eq defined_attributes.attributes());
    }

    #[conformance_test]
    pub fn opener_succeeds_when_attributes_do_match<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Take a number".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"3. Just have a".try_into().unwrap(),
                &"lick on the toad".try_into().unwrap(),
            )
            .unwrap();

        let _sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require(
                    &"1. Hello".try_into().unwrap(),
                    &"Hypnotoad".try_into().unwrap(),
                )
                .unwrap()
                .require(
                    &"1. Hello".try_into().unwrap(),
                    &"Take a number".try_into().unwrap(),
                )
                .unwrap()
                .require(
                    &"3. Just have a".try_into().unwrap(),
                    &"lick on the toad".try_into().unwrap(),
                )
                .unwrap(),
        );

        assert_that!(sut_open, is_ok);
        let sut_open = sut_open.unwrap();

        assert_that!(sut_open.attributes(), eq defined_attributes.attributes());
    }

    #[conformance_test]
    pub fn opener_fails_when_attribute_value_does_not_match<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap();
        let _sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require(
                    &"1. Hello".try_into().unwrap(),
                    &"lick on the toad".try_into().unwrap(),
                )
                .unwrap(),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[conformance_test]
    pub fn opener_fails_when_attribute_key_does_not_exist<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap();
        let _sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require(
                    &"Whatever".try_into().unwrap(),
                    &"lick on the toad".try_into().unwrap(),
                )
                .unwrap(),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[conformance_test]
    pub fn opener_fails_when_attribute_value_does_not_exist<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Number Two".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap();
        let _sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require(
                    &"1. Hello".try_into().unwrap(),
                    &"lick on the toad".try_into().unwrap(),
                )
                .unwrap()
                .require(
                    &"1. Hello".try_into().unwrap(),
                    &"Number Eight".try_into().unwrap(),
                )
                .unwrap(),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[conformance_test]
    pub fn opener_fails_when_attribute_required_key_does_not_exist<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap();
        let _sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require_key(&"i do not exist".try_into().unwrap())
                .unwrap(),
        );

        assert_that!(sut_open, is_err);
        Factory::assert_attribute_error(sut_open.err().unwrap());
    }

    #[conformance_test]
    pub fn opener_succeeds_when_attribute_required_key_does_exist<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let defined_attributes = AttributeSpecifier::new()
            .define(
                &"1. Hello".try_into().unwrap(),
                &"Hypnotoad".try_into().unwrap(),
            )
            .unwrap()
            .define(
                &"2. No more".try_into().unwrap(),
                &"Coffee".try_into().unwrap(),
            )
            .unwrap();
        let _sut_create = test
            .create(&node_1, &service_name, &defined_attributes)
            .unwrap();

        let sut_open = test.open(
            &node_2,
            &service_name,
            &AttributeVerifier::new()
                .require_key(&"2. No more".try_into().unwrap())
                .unwrap(),
        );

        assert_that!(sut_open, is_ok);
    }

    #[conformance_test]
    pub fn details_error_display_works<Sut: Service, Factory: SutFactory<Sut>>() {
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

    #[conformance_test]
    pub fn list_error_display_works<Sut: Service, Factory: SutFactory<Sut>>() {
        assert_that!(format!("{}", ServiceListError::InsufficientPermissions), eq
    "ServiceListError::InsufficientPermissions");

        assert_that!(format!("{}", ServiceListError::InternalError), eq
    "ServiceListError::InternalError");
    }

    #[conformance_test]
    pub fn list_services_works<Sut: Service, Factory: SutFactory<Sut>>() {
        const NUMBER_OF_SERVICES: usize = 16;
        let test = Factory::new();

        let mut services = vec![];
        let mut service_hashs = vec![];
        let mut nodes = vec![];
        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();
            let node = test.context().create_node();
            let sut = test
                .create(&node, &service_name, &AttributeSpecifier::new())
                .unwrap();

            service_hashs.push(*sut.service_hash());
            services.push(sut);
            nodes.push(node);
        }

        let mut listed_services = vec![];
        let result = Sut::list(test.context().config(), |service| {
            listed_services.push(*service.static_details.service_hash());
            CallbackProgression::Continue
        });
        assert_that!(result, is_ok);

        for s in listed_services {
            assert_that!(service_hashs, contains s);
        }
    }

    #[conformance_test]
    pub fn list_services_stops_when_callback_progression_states_stop<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        const NUMBER_OF_SERVICES: usize = 16;
        let test = Factory::new();
        let node = test.context().create_node();

        let mut services = vec![];
        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();
            let sut = test
                .create(&node, &service_name, &AttributeSpecifier::new())
                .unwrap();

            services.push(sut);
        }

        let mut service_counter = 0;
        let result = Sut::list(test.context().config(), |_service| {
            service_counter += 1;
            CallbackProgression::Stop
        });
        assert_that!(result, is_ok);
        assert_that!(service_counter, eq 1);
    }

    #[cfg(not(target_os = "windows"))]
    // disabled since the windows defender interfers and causes ERROR_ACCESS_DENIED failures on the platform when it locks file for scanning
    #[conformance_test]
    pub fn concurrent_service_creation_and_listing_works<Sut: Service, Factory: SutFactory<Sut>>() {
        let test =
            Factory::new_with_custom_watchdog(Watchdog::new_with_timeout(Duration::from_secs(120)));
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        const NUMBER_OF_ITERATIONS: usize = 40;
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(number_of_creators as _)
            .create(&handle)
            .unwrap();

        thread_scope(|s| {
            for _ in 0..number_of_creators {
                s.thread_builder().spawn(|| {
                    let node = test.context().create_node();
                    barrier.wait();

                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let service_name = generate_service_name();
                        let sut = test
                            .create(&node, &service_name, &AttributeSpecifier::new())
                            .unwrap();

                        let mut found_me = false;
                        let result = Sut::list(test.context().config(), |s| {
                            if sut.service_hash() == s.static_details.service_hash() {
                                found_me = true;
                            }
                            CallbackProgression::Continue
                        });

                        assert_that!(result, is_ok);
                        assert_that!(found_me, eq true);
                    }
                })?;
            }

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn concurrent_node_attaching_to_service_and_listing_works<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test =
            Factory::new_with_custom_watchdog(Watchdog::new_with_timeout(Duration::from_secs(120)));
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        const NUMBER_OF_ITERATIONS: usize = 30;
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(number_of_creators as _)
            .create(&handle)
            .unwrap();

        let main_node = test.context().create_node();
        let service_name = generate_service_name();
        let attributes = AttributeVerifier::new();
        let _service = test.create(&main_node, &service_name, &AttributeSpecifier::new());

        thread_scope(|s| {
            for _ in 0..number_of_creators {
                s.thread_builder().spawn(|| {
                    barrier.wait();

                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let node = test.context().create_node();
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
                })?;
            }

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn concurrent_node_attaching_to_service_and_details_node_listing_works<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test =
            Factory::new_with_custom_watchdog(Watchdog::new_with_timeout(Duration::from_secs(120)));
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 4);
        const NUMBER_OF_ITERATIONS: usize = 30;
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(number_of_creators as _)
            .create(&handle)
            .unwrap();

        let main_node = test.context().create_node();
        let service_name = generate_service_name();
        let attributes = AttributeVerifier::new();
        let _service = test.create(&main_node, &service_name, &AttributeSpecifier::new());

        thread_scope(|s| {
            for _ in 0..number_of_creators {
                s.thread_builder().spawn(|| {
                    barrier.wait();

                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let node = test.context().create_node();
                        let _service = test.open(&node, &service_name, &attributes).unwrap();

                        let service_details = Sut::details(
                            &service_name,
                            test.context().config(),
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
                })?;
            }

            Ok(())
        })
        .unwrap();
    }

    #[conformance_test]
    pub fn node_listing_works<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        const NUMBER_OF_NODES: usize = 5;

        let main_node = test.context().create_node();
        let service_name = generate_service_name();
        let attributes = AttributeVerifier::new();
        let main_service = test
            .create(&main_node, &service_name, &AttributeSpecifier::new())
            .unwrap();

        let mut nodes = vec![];
        let mut node_ids = vec![];
        let mut services = vec![];
        node_ids.push(*main_node.id());

        let get_registered_node_ids = |service: &Factory::Factory| {
            let mut registered_node_ids = vec![];
            service
                .nodes(|node_state| {
                    match node_state {
                        NodeState::Alive(view) => registered_node_ids.push(*view.id()),
                        NodeState::Dead(view) => registered_node_ids.push(*view.id()),
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
            let node = test.context().create_node();
            let service = test.open(&node, &service_name, &attributes).unwrap();

            let registered_node_ids = get_registered_node_ids(&service);

            node_ids.push(*node.id());
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

    #[conformance_test]
    pub fn node_can_open_same_service_without_limits<Sut: Service, Factory: SutFactory<Sut>>() {
        let test = Factory::new();
        let service_name = generate_service_name();
        const REPETITIONS: usize = 128;

        let node = test.context().create_node();
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

    #[conformance_test]
    pub fn service_hash_is_equal_in_within_all_opened_instances<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let node = test.context().create_node();

        let sut = test
            .create(&node, &service_name, &AttributeSpecifier::new())
            .unwrap();
        let sut2 = test
            .open(&node, &service_name, &AttributeVerifier::new())
            .unwrap();

        assert_that!(sut.service_hash(), eq sut2.service_hash());
    }

    #[conformance_test]
    pub fn service_can_be_forcefully_removed_and_recreated_and_opened_again<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let test = Factory::new();
        let service_name = generate_service_name();
        let dead_node = test.context().create_node();
        let node = test.context().create_node();

        let dead_sut = test
            .create(&dead_node, &service_name, &AttributeSpecifier::new())
            .unwrap();
        dead_sut.abandon();
        dead_node.abandon();

        assert_that!(
            unsafe { node.force_remove_service(&service_name, Factory::messaging_pattern()) },
            is_ok
        );

        let create_sut = test.create(&node, &service_name, &AttributeSpecifier::new());
        assert_that!(create_sut, is_ok);

        let open_sut = test.open(&node, &service_name, &AttributeVerifier::new());
        assert_that!(open_sut, is_ok);
    }

    #[conformance_test]
    pub fn dead_nodes_are_cleaned_up_when_cleanup_on_open_is_active<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let mut test = Factory::new();
        let service_name = generate_service_name();
        test.context_mut()
            .config_mut()
            .global
            .service
            .cleanup_dead_nodes_on_open = true;

        let dead_node = test.context().create_node();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();
        let node_ids = vec![*node_1.id(), *node_2.id()];

        let dead_service = test
            .create(&dead_node, &service_name, &AttributeSpecifier::new())
            .unwrap();
        let _service_1 = test
            .open(&node_1, &service_name, &AttributeVerifier::new())
            .unwrap();

        dead_service.abandon();
        dead_node.abandon();

        let service_2 = test
            .open(&node_2, &service_name, &AttributeVerifier::new())
            .unwrap();

        let mut counter = 0;
        service_2
            .nodes(|node_state| {
                counter += 1;
                if let NodeState::Alive(node_state) = node_state {
                    assert_that!(node_ids, contains * node_state.id());
                } else {
                    assert_that!(shall_never_be_reached);
                }
                CallbackProgression::Continue
            })
            .unwrap();

        assert_that!(counter, eq 2);
    }

    #[conformance_test]
    pub fn dead_nodes_are_cleaned_up_before_opening_the_service<
        Sut: Service,
        Factory: SutFactory<Sut>,
    >() {
        let mut test = Factory::new();
        let service_name = generate_service_name();
        test.context_mut()
            .config_mut()
            .global
            .service
            .cleanup_dead_nodes_on_open = true;
        test.set_max_number_of_nodes(2);

        let dead_node = test.context().create_node();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();

        let dead_service = test
            .create(&dead_node, &service_name, &AttributeSpecifier::new())
            .unwrap();
        let _service_1 = test
            .open(&node_1, &service_name, &AttributeVerifier::new())
            .unwrap();

        dead_service.abandon();
        dead_node.abandon();

        let service_2 = test.open(&node_2, &service_name, &AttributeVerifier::new());
        assert_that!(service_2, is_ok);
    }

    #[conformance_test]
    pub fn no_cleanup_when_on_open_cleanup_is_disabled<Sut: Service, Factory: SutFactory<Sut>>() {
        let mut test = Factory::new();
        let service_name = generate_service_name();
        test.set_max_number_of_nodes(2);

        let dead_node = test.context().create_node();
        let node_1 = test.context().create_node();
        let node_2 = test.context().create_node();

        let dead_service = test
            .create(&dead_node, &service_name, &AttributeSpecifier::new())
            .unwrap();
        let _service_1 = test
            .open(&node_1, &service_name, &AttributeVerifier::new())
            .unwrap();

        dead_service.abandon();
        dead_node.abandon();

        let service_2 = test.open(&node_2, &service_name, &AttributeVerifier::new());
        assert_that!(service_2, is_err);
    }
}
