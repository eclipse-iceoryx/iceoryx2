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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Barrier;

    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::event::{EventCreateError, EventOpenError};
    use iceoryx2::service::builder::publish_subscribe::{
        PublishSubscribeCreateError, PublishSubscribeOpenError,
    };
    use iceoryx2::service::port_factory::{event, publish_subscribe};
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

    trait SutFactory {
        type Factory;
        type CreateError: std::fmt::Debug;
        type OpenError: std::fmt::Debug;

        fn create(service_name: &ServiceName) -> Result<Self::Factory, Self::CreateError>;
        fn open(service_name: &ServiceName) -> Result<Self::Factory, Self::OpenError>;

        fn assert_create_error(error: Self::CreateError);
        fn assert_open_error(error: Self::OpenError);
    }

    impl<Sut: Service> SutFactory for publish_subscribe::PortFactory<Sut, u64> {
        type Factory = publish_subscribe::PortFactory<Sut, u64>;
        type CreateError = PublishSubscribeCreateError;
        type OpenError = PublishSubscribeOpenError;

        fn create(service_name: &ServiceName) -> Result<Self::Factory, Self::CreateError> {
            Sut::new(&service_name).publish_subscribe().create::<u64>()
        }

        fn open(service_name: &ServiceName) -> Result<Self::Factory, Self::OpenError> {
            Sut::new(&service_name).publish_subscribe().open::<u64>()
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
                    PublishSubscribeOpenError::PermissionDenied,
                    PublishSubscribeOpenError::UnableToOpenDynamicServiceInformation,
                ])
            );
        }
    }

    impl<Sut: Service> SutFactory for event::PortFactory<Sut> {
        type Factory = event::PortFactory<Sut>;
        type CreateError = EventCreateError;
        type OpenError = EventOpenError;

        fn create(service_name: &ServiceName) -> Result<Self::Factory, Self::CreateError> {
            Sut::new(&service_name).event().create()
        }

        fn open(service_name: &ServiceName) -> Result<Self::Factory, Self::OpenError> {
            Sut::new(&service_name).event().open()
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
                    EventOpenError::PermissionDenied,
                    EventOpenError::UnableToOpenDynamicServiceInformation,
                ])
            );
        }
    }

    #[test]
    fn same_name_with_different_messaging_pattern_is_allowed<Sut: Service, Factory: SutFactory>() {
        let service_name = generate_name();
        let sut_pub_sub = Sut::new(&service_name).publish_subscribe().create::<u64>();
        assert_that!(sut_pub_sub, is_ok);
        let sut_pub_sub = sut_pub_sub.unwrap();

        let sut_event = Sut::new(&service_name).event().create();
        assert_that!(sut_event, is_ok);
        let sut_event = sut_event.unwrap();

        let sut_subscriber = sut_pub_sub.subscriber().create().unwrap();
        let sut_publisher = sut_pub_sub.publisher().create().unwrap();

        let mut sut_listener = sut_event.listener().create().unwrap();
        let sut_notifier = sut_event.notifier().create().unwrap();

        const SAMPLE_VALUE: u64 = 891231211;
        sut_publisher.send_copy(SAMPLE_VALUE).unwrap();
        let received_sample = sut_subscriber.receive().unwrap().unwrap();
        assert_that!(*received_sample, eq(SAMPLE_VALUE));

        const EVENT_ID: EventId = EventId::new(31);
        sut_notifier.notify_with_custom_event_id(EVENT_ID).unwrap();
        let received_event = sut_listener.try_wait().unwrap();
        assert_that!(received_event, len(1));
        assert_that!(received_event[0], eq(EVENT_ID));
    }

    #[test]
    fn concurent_creating_services_with_unique_names_is_successful<
        Sut: Service,
        Factory: SutFactory,
    >() {
        let _watch_dog = Watchdog::new();
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024) * 2;
        const NUMBER_OF_ITERATIONS: usize = 50;

        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_threads {
                threads.push(s.spawn(|| {
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        barrier_enter.wait();

                        let service_name = generate_name();
                        let _sut = Factory::create(&service_name).unwrap();

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
        Factory: SutFactory,
    >() {
        let _watch_dog = Watchdog::new();
        let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024) * 2;
        const NUMBER_OF_ITERATIONS: usize = 50;

        let success_counter = AtomicU64::new(0);
        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);
        let service_name = generate_name();

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_threads {
                threads.push(s.spawn(|| {
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        barrier_enter.wait();

                        let sut = Factory::create(&service_name);
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
        Factory: SutFactory,
    >() {
        let _watch_dog = Watchdog::new();
        const NUMBER_OF_CLOSE_THREADS: usize = 1;
        let number_of_open_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024) * 2;
        let number_of_threads = NUMBER_OF_CLOSE_THREADS + number_of_open_threads;
        const NUMBER_OF_ITERATIONS: usize = 50;

        let barrier_enter = Barrier::new(number_of_threads);
        let barrier_exit = Barrier::new(number_of_threads);
        let service_name = generate_name();

        std::thread::scope(|s| {
            let mut threads = vec![];
            threads.push(s.spawn(|| {
                for _ in 0..NUMBER_OF_ITERATIONS {
                    let sut = Factory::create(&service_name).unwrap();
                    barrier_enter.wait();

                    drop(sut);

                    barrier_exit.wait();
                }
            }));

            for _ in 0..number_of_open_threads {
                threads.push(s.spawn(|| {
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        barrier_enter.wait();

                        let sut = Factory::open(&service_name);
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

    mod zero_copy {
        use iceoryx2::service::port_factory::event::PortFactory as EventPortFactory;
        use iceoryx2::service::port_factory::publish_subscribe::PortFactory as PubSubPortFactory;
        use iceoryx2::service::zero_copy::Service;

        #[instantiate_tests(<Service, EventPortFactory::<Service>>)]
        mod event {}
        #[instantiate_tests(<Service, PubSubPortFactory::<Service, u64>>)]
        mod publish_subscribe {}
    }

    mod process_local {
        use iceoryx2::service::port_factory::event::PortFactory as EventPortFactory;
        use iceoryx2::service::port_factory::publish_subscribe::PortFactory as PubSubPortFactory;
        use iceoryx2::service::process_local::Service;

        #[instantiate_tests(<Service, EventPortFactory::<Service>>)]
        mod event {}
        #[instantiate_tests(<Service, PubSubPortFactory::<Service, u64>>)]
        mod publish_subscribe {}
    }
}
