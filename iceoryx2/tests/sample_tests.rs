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
mod sample {
    use iceoryx2::port::publisher::Publisher;
    use iceoryx2::port::subscriber::Subscriber;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::publish_subscribe::PublishSubscribeCreateError;
    use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    struct TestContext<Sut: Service> {
        node: Node<Sut>,
        service_name: ServiceName,
        service: PortFactory<Sut, u64, ()>,
        publisher_1: Publisher<Sut, u64, ()>,
        publisher_2: Publisher<Sut, u64, ()>,
        subscriber: Subscriber<Sut, u64, ()>,
    }

    impl<Sut: Service> TestContext<Sut> {
        fn new(config: &Config) -> Self {
            let node = NodeBuilder::new().config(config).create::<Sut>().unwrap();
            let service_name = generate_name();
            let service = node
                .service_builder(&service_name)
                .publish_subscribe::<u64>()
                .max_publishers(2)
                .max_subscribers(1)
                .create()
                .unwrap();

            let publisher_1 = service.publisher_builder().create().unwrap();

            let publisher_2 = service.publisher_builder().create().unwrap();

            let subscriber = service.subscriber_builder().create().unwrap();

            Self {
                node,
                service_name,
                service,
                publisher_1,
                publisher_2,
                subscriber,
            }
        }
    }

    #[test]
    fn origin_is_tracked_correctly<Sut: Service>() {
        let config = generate_isolated_config();
        let test_context = TestContext::<Sut>::new(&config);

        assert_that!(test_context.publisher_1.send_copy(123), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(sample.origin(), eq test_context.publisher_1.id());

        assert_that!(test_context.publisher_2.send_copy(123), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(sample.origin(), eq test_context.publisher_2.id());
    }

    #[test]
    fn sample_of_dropped_service_does_block_new_service_creation<Sut: Service>() {
        let config = generate_isolated_config();
        let test_context = TestContext::<Sut>::new(&config);

        let service_name = test_context.service_name.clone();

        assert_that!(test_context.publisher_1.send_copy(5), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap();
        assert_that!(sample, is_some);

        drop(test_context);

        let test_context = TestContext::<Sut>::new(&config);

        let result = test_context
            .node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create();
        assert_that!(result.err(), eq Some(PublishSubscribeCreateError::AlreadyExists));
    }

    #[test]
    fn when_everything_is_dropped_the_sample_can_still_be_consumed<Sut: Service>() {
        let config = generate_isolated_config();
        let test_context = TestContext::<Sut>::new(&config);

        let sut = test_context.service;
        let publisher_1 = test_context.publisher_1;
        let publisher_2 = test_context.publisher_2;
        let subscriber = test_context.subscriber;

        drop(sut);

        const PAYLOAD: u64 = 8761238679123;

        assert_that!(publisher_1.send_copy(PAYLOAD), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();

        drop(subscriber);
        drop(publisher_1);
        drop(publisher_2);

        assert_that!(*sample, eq PAYLOAD);
    }

    #[test]
    fn sample_received_from_dropped_publisher_does_not_block_new_publishers<Sut: Service>() {
        let config = generate_isolated_config();
        let test_context = TestContext::<Sut>::new(&config);
        const PAYLOAD_1: u64 = 123554;

        let publisher = test_context.publisher_1;

        assert_that!(publisher.send_copy(PAYLOAD_1), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap().unwrap();

        drop(publisher);

        const PAYLOAD_2: u64 = 123981235645;

        let publisher = test_context.service.publisher_builder().create().unwrap();
        assert_that!(publisher.send_copy(PAYLOAD_2), eq Ok(1));
        assert_that!(*sample, eq PAYLOAD_1);
        let sample_2 = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(*sample_2, eq PAYLOAD_2);
    }

    #[test]
    fn sample_from_dropped_subscriber_does_not_block_new_subscribers<Sut: Service>() {
        let mut config = generate_isolated_config();
        config.defaults.publish_subscribe.publisher_history_size = 1;
        let test_context = TestContext::<Sut>::new(&config);
        const PAYLOAD_1: u64 = 7781123554;

        let subscriber = test_context.subscriber;

        assert_that!(test_context.publisher_1.send_copy(PAYLOAD_1), eq Ok(1));
        let _sample = subscriber.receive().unwrap().unwrap();

        drop(subscriber);

        const PAYLOAD_2: u64 = 123666645;

        let subscriber = test_context.service.subscriber_builder().create().unwrap();
        assert_that!(test_context.publisher_1.send_copy(PAYLOAD_2), eq Ok(1));
        let sample_1 = subscriber.receive().unwrap().unwrap();
        let sample_2 = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample_1, eq PAYLOAD_1);
        assert_that!(*sample_2, eq PAYLOAD_2);
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
