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
mod publisher {
    use std::time::{Duration, Instant};

    use iceoryx2::port::publisher::PublisherLoanError;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::publisher::UnableToDeliverStrategy;
    use iceoryx2::service::{service_name::ServiceName, Service};
    use iceoryx2_bb_posix::barrier::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;

    type TestResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;

    const TIMEOUT: Duration = Duration::from_millis(25);

    fn generate_name() -> TestResult<ServiceName> {
        Ok(ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))?)
    }

    const COMPLEX_TYPE_DEFAULT_VALUE: u64 = 872379237;

    #[derive(Debug)]
    #[repr(C)]
    struct ComplexType {
        data: u64,
    }

    impl Default for ComplexType {
        fn default() -> Self {
            ComplexType {
                data: COMPLEX_TYPE_DEFAULT_VALUE,
            }
        }
    }

    #[test]
    fn publisher_loan_and_send_sample_works<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let sample = sut.loan()?;

        assert_that!(sample.send(), is_ok);

        Ok(())
    }

    #[test]
    fn publisher_loan_initializes_sample_with_default<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<ComplexType>()
            .create()?;

        let publisher = service.publisher().create()?;
        let sut = publisher.loan()?;

        assert_that!(sut.payload().data, eq COMPLEX_TYPE_DEFAULT_VALUE);

        Ok(())
    }

    #[test]
    fn publisher_loan_slice_initializes_sample_with_default<Sut: Service>() -> TestResult<()> {
        const NUMBER_OF_ELEMENTS: usize = 120;
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<[ComplexType]>()
            .create()?;

        let publisher = service
            .publisher()
            .max_slice_len(NUMBER_OF_ELEMENTS)
            .create()?;
        let sut = publisher.loan_slice(NUMBER_OF_ELEMENTS)?;

        for i in 0..NUMBER_OF_ELEMENTS {
            assert_that!(sut.payload()[i].data, eq COMPLEX_TYPE_DEFAULT_VALUE);
        }

        Ok(())
    }

    #[test]
    fn publisher_loan_slice_up_to_max_elements_works<Sut: Service>() -> TestResult<()> {
        const NUMBER_OF_ELEMENTS: usize = 125;
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<[ComplexType]>()
            .create()?;

        let publisher = service
            .publisher()
            .max_slice_len(NUMBER_OF_ELEMENTS)
            .create()?;

        for i in 0..NUMBER_OF_ELEMENTS {
            let sut = publisher.loan_slice(i)?;
            assert_that!(sut.payload().len(), eq i);
        }

        Ok(())
    }

    #[test]
    fn publisher_loan_slice_more_than_max_elements_fails<Sut: Service>() -> TestResult<()> {
        const NUMBER_OF_ELEMENTS: usize = 125;
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<[ComplexType]>()
            .create()?;

        let publisher = service
            .publisher()
            .max_slice_len(NUMBER_OF_ELEMENTS)
            .create()?;

        let sut = publisher.loan_slice(NUMBER_OF_ELEMENTS + 1);
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq PublisherLoanError::ExceedsMaxLoanSize);

        Ok(())
    }

    #[test]
    fn publisher_loan_unit_and_send_sample_works<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let sample = sut.loan_uninit()?.write_payload(42);

        assert_that!(sample.send(), is_ok);

        Ok(())
    }

    #[test]
    fn publisher_can_borrow_multiple_sample_at_once<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.publisher().max_loaned_samples(4).create()?;

        let sample1 = sut.loan_uninit()?.write_payload(1);
        let sample2 = sut.loan_uninit()?.write_payload(2);
        let sample3 = sut.loan_uninit()?.write_payload(3);

        let subscriber = service.subscriber().create()?;

        assert_that!(sut.send_copy(4), is_ok);
        assert_that!(sample3.send(), is_ok);
        drop(sample2);
        drop(sample1);

        let r = subscriber.receive()?;
        assert_that!(r, is_some);
        assert_that!( *r.unwrap(), eq 4);
        let r = subscriber.receive()?;
        assert_that!(r, is_some);
        assert_that!( *r.unwrap(), eq 3);

        Ok(())
    }

    #[test]
    fn publisher_max_loaned_samples_works<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let _sample1 = sut.loan_uninit()?;
        let _sample2 = sut.loan_uninit()?;

        let sample3 = sut.loan_uninit();
        assert_that!(sample3, is_err);
        assert_that!(sample3.err().unwrap(), eq PublisherLoanError::ExceedsMaxLoanedChunks);

        Ok(())
    }

    #[test]
    fn publisher_sending_sample_reduces_loan_counter<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let _sample1 = sut.loan_uninit()?;
        let sample2 = sut.loan_uninit()?.write_payload(2);

        assert_that!(sample2.send(), is_ok);

        let _sample3 = sut.loan_uninit();
        let sample4 = sut.loan_uninit();
        assert_that!(sample4, is_err);
        assert_that!(sample4.err().unwrap(), eq PublisherLoanError::ExceedsMaxLoanedChunks);

        Ok(())
    }

    #[test]
    fn publisher_dropping_sample_reduces_loan_counter<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .create()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let _sample1 = sut.loan_uninit()?;
        let sample2 = sut.loan_uninit()?;

        drop(sample2);

        let _sample3 = sut.loan_uninit();
        let sample4 = sut.loan_uninit();
        assert_that!(sample4, is_err);
        assert_that!(sample4.err().unwrap(), eq PublisherLoanError::ExceedsMaxLoanedChunks);

        Ok(())
    }

    #[test]
    fn publisher_block_when_unable_to_deliver_blocks<Sut: Service>() -> TestResult<()> {
        let _watchdog = Watchdog::new();
        let service_name = generate_name()?;
        let node = NodeBuilder::new().create::<Sut>().unwrap();
        let service = node
            .service(&service_name)
            .publish_subscribe::<u64>()
            .subscriber_max_buffer_size(1)
            .enable_safe_overflow(false)
            .create()?;

        let sut = service
            .publisher()
            .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
            .create()?;

        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            s.spawn(|| {
                let service = node
                    .service(&service_name)
                    .publish_subscribe::<u64>()
                    .subscriber_max_buffer_size(1)
                    .open()
                    .unwrap();

                let subscriber = service.subscriber().create().unwrap();
                let receive_sample = || loop {
                    if let Some(sample) = subscriber.receive().unwrap() {
                        return sample;
                    }
                };

                barrier.wait();
                std::thread::sleep(TIMEOUT);
                let sample_1 = receive_sample();
                std::thread::sleep(TIMEOUT);
                let sample_2 = receive_sample();

                assert_that!(*sample_1, eq 8192);
                assert_that!(*sample_2, eq 2);
            });

            barrier.wait();
            let now = Instant::now();
            sut.send_copy(8192).unwrap();
            sut.send_copy(2).unwrap();
            assert_that!(now.elapsed(), time_at_least TIMEOUT);
        });

        Ok(())
    }

    #[instantiate_tests(<iceoryx2::service::zero_copy::Service>)]
    mod zero_copy {}

    #[instantiate_tests(<iceoryx2::service::process_local::Service>)]
    mod process_local {}
}
