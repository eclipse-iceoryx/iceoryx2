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
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn origin_is_tracked_correctly<Sut: Service>() {
        let service_name = generate_name();
        let sut = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()
            .unwrap();

        let publisher_1 = sut.publisher().create().unwrap();
        let publisher_2 = sut.publisher().create().unwrap();
        let subscriber = sut.subscriber().create().unwrap();

        assert_that!(publisher_1.send_copy(123), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();
        assert_that!(sample.origin(), eq publisher_1.id());

        assert_that!(publisher_2.send_copy(123), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();
        assert_that!(sample.origin(), eq publisher_2.id());
    }

    #[test]
    fn sample_of_dropped_service_does_not_block_new_service_creation<Sut: Service>() {
        let service_name = generate_name();
        let sut = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()
            .unwrap();

        let subscriber = sut.subscriber().create().unwrap();
        let publisher = sut.publisher().create().unwrap();

        assert_that!(publisher.send_copy(5), eq Ok(1));
        let sample = subscriber.receive().unwrap();
        assert_that!(sample, is_some);

        drop(sut);
        drop(subscriber);
        drop(publisher);

        assert_that!(
            Sut::new(&service_name).publish_subscribe().create::<u64>(),
            is_ok
        );
    }

    #[test]
    fn when_everything_is_dropped_the_sample_can_still_be_consumed<Sut: Service>() {
        let service_name = generate_name();
        let sut = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()
            .unwrap();

        let subscriber = sut.subscriber().create().unwrap();
        let publisher = sut.publisher().create().unwrap();

        drop(sut);

        const PAYLOAD: u64 = 8761238679123;

        assert_that!(publisher.send_copy(PAYLOAD), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();

        drop(subscriber);
        drop(publisher);

        assert_that!(*sample, eq PAYLOAD);
    }

    #[test]
    fn sample_received_from_dropped_publisher_does_not_block_new_publishers<Sut: Service>() {
        let service_name = generate_name();
        let sut = Sut::new(&service_name)
            .publish_subscribe()
            .max_publishers(1)
            .create::<u64>()
            .unwrap();

        let subscriber = sut.subscriber().create().unwrap();
        let publisher = sut.publisher().create().unwrap();

        assert_that!(publisher.send_copy(123554), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();

        drop(publisher);

        const PAYLOAD: u64 = 123981235645;

        let publisher = sut.publisher().create().unwrap();
        assert_that!(publisher.send_copy(PAYLOAD), eq Ok(1));
        assert_that!(*sample, eq 123554);
        let sample_2 = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample_2, eq PAYLOAD);
    }

    #[test]
    fn sample_from_dropped_subscriber_does_not_block_new_subscribers<Sut: Service>() {
        let service_name = generate_name();
        let sut = Sut::new(&service_name)
            .publish_subscribe()
            .max_subscribers(1)
            .history_size(0)
            .create::<u64>()
            .unwrap();

        let subscriber = sut.subscriber().create().unwrap();
        let publisher = sut.publisher().create().unwrap();

        assert_that!(publisher.send_copy(1234), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();

        drop(subscriber);

        const PAYLOAD: u64 = 123666645;

        let subscriber = sut.subscriber().create().unwrap();
        assert_that!(publisher.send_copy(PAYLOAD), eq Ok(1));
        let sample_2 = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample, eq 1234);
        assert_that!(*sample_2, eq PAYLOAD);
    }

    #[instantiate_tests(<iceoryx2::service::zero_copy::Service>)]
    mod zero_copy {}

    #[instantiate_tests(<iceoryx2::service::process_local::Service>)]
    mod process_local {}
}
