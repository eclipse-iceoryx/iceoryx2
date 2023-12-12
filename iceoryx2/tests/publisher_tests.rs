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

    use iceoryx2::port::publisher::LoanError;
    use iceoryx2::service::port_factory::publisher::UnableToDeliverStrategy;
    use iceoryx2::service::{service_name::ServiceName, Service};
    use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle};
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    type TestResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;

    const TIMEOUT: Duration = Duration::from_millis(25);

    fn generate_name() -> TestResult<ServiceName> {
        Ok(ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))?)
    }

    #[test]
    fn publisher_loan_and_send_sample_works<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let sample = sut.loan()?;

        assert_that!(sut.send(sample), is_ok);

        Ok(())
    }

    #[test]
    fn publisher_loan_unit_and_send_sample_works<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let sample = sut.loan_uninit()?.write_payload(42);

        assert_that!(sut.send(sample), is_ok);

        Ok(())
    }

    #[test]
    fn publisher_can_borrow_multiple_sample_at_once<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()?;

        let sut = service.publisher().max_loaned_samples(4).create()?;

        let sample1 = sut.loan_uninit()?.write_payload(1);
        let sample2 = sut.loan_uninit()?.write_payload(2);
        let sample3 = sut.loan_uninit()?.write_payload(3);

        let subscriber = service.subscriber().create()?;

        assert_that!(sut.send_copy(4), is_ok);
        assert_that!(sut.send(sample3), is_ok);
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
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let _sample1 = sut.loan_uninit()?;
        let _sample2 = sut.loan_uninit()?;

        let sample3 = sut.loan_uninit();
        assert_that!(sample3, is_err);
        assert_that!(sample3.err().unwrap(), eq LoanError::ExceedsMaxLoanedChunks);

        Ok(())
    }

    #[test]
    fn publisher_sending_sample_reduces_loan_counter<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let _sample1 = sut.loan_uninit()?;
        let sample2 = sut.loan_uninit()?.write_payload(2);

        assert_that!(sut.send(sample2), is_ok);

        let _sample3 = sut.loan_uninit();
        let sample4 = sut.loan_uninit();
        assert_that!(sample4, is_err);
        assert_that!(sample4.err().unwrap(), eq LoanError::ExceedsMaxLoanedChunks);

        Ok(())
    }

    #[test]
    fn publisher_dropping_sample_reduces_loan_counter<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .create::<u64>()?;

        let sut = service.publisher().max_loaned_samples(2).create()?;

        let _sample1 = sut.loan_uninit()?;
        let sample2 = sut.loan_uninit()?;

        drop(sample2);

        let _sample3 = sut.loan_uninit();
        let sample4 = sut.loan_uninit();
        assert_that!(sample4, is_err);
        assert_that!(sample4.err().unwrap(), eq LoanError::ExceedsMaxLoanedChunks);

        Ok(())
    }

    //TODO iox2-#44
    #[ignore]
    #[test]
    fn publisher_block_when_unable_to_deliver_blocks<Sut: Service>() -> TestResult<()> {
        let service_name = generate_name()?;
        let service = Sut::new(&service_name)
            .publish_subscribe()
            .subscriber_max_buffer_size(1)
            .enable_safe_overflow(false)
            .create::<u64>()?;

        let sut = service
            .publisher()
            .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
            .create()?;

        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            s.spawn(|| {
                let service = Sut::new(&service_name)
                    .publish_subscribe()
                    .subscriber_max_buffer_size(1)
                    .open::<u64>()
                    .unwrap();

                let subscriber = service.subscriber().create().unwrap();
                barrier.wait();
                std::thread::sleep(TIMEOUT);
                let sample_1 = subscriber.receive().unwrap().unwrap();
                std::thread::sleep(TIMEOUT);
                let sample_2 = subscriber.receive().unwrap().unwrap();

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
